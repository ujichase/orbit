/*
File: datapath.sv
Author: Chase Ruskin
Details:
    Datapath implementation for counting the number of bits asserted ('1') for 
    a generic-width input.
*/

module datapath
#(
    parameter int TEMP = $clog2(24),
    parameter int MAC = `NUM_BITS,
    parameter int WIDTH = 8
)
(
    input logic clk,
    input logic rst,
    input logic [WIDTH-1:0] in,
    input logic en_num,
    input logic en_ctr,
    input logic sel_num,
    input logic sel_ctr,

    output logic is_num_zero,
    output logic [$clog2(WIDTH+1)-1:0] out
);
    // --- signals
    logic [WIDTH-1:0] num_r, next_num;
    logic [$bits(out)-1:0] ctr_r, next_ctr;

    logic [$bits(out)-1:0] ctr_plus_one; 
    logic [WIDTH-1:0] num_one_less_bit;

    // --- registers
    always_ff @(posedge clk or posedge rst) begin
        // asynchronous active-high reset
        if (rst == 1'b1) begin
            num_r <= '0;
            ctr_r <= '0;
        end else begin
            // synchronous enable signals controlling the registers from `fsm`
            if (en_num == 1'b1) begin num_r <= next_num; end
            if (en_ctr == 1'b1) begin ctr_r <= next_ctr; end
        end
    end

    // --- combinational logic

    // decrement a bit from the current number bit encoding
    assign num_one_less_bit = num_r & (num_r - ($bits(num_r))'(1));

    // mux to determine what value to store into internal number register
    // 0 -> in
    // 1 -> num_one_less_bit
    assign next_num = (sel_num == 1'b1) ? num_one_less_bit : in;

    // increment the current count
    assign ctr_plus_one = ctr_r + ($bits(ctr_r))'(1);

    // mux to determine what value to store into the internal counter register
    // 0 -> '0
    // 1 -> ctr_plus_one
    assign next_ctr = (sel_ctr == 1'b1) ? ctr_plus_one : '0;

    // conditional flag to send to `fsm` whether the algorithm should halt or continue
    assign is_num_zero = (num_r == '0) ? 1'b1 : 1'b0;

    // continuously drive the output 
    // @note: assumes `fsm` handles declaring when the output is 'valid'
    assign out = ctr_r;

endmodule
