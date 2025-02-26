
module \sv_to_vhd 
#(
    parameter int WORD_WIDTH = 32
) (
    input clk,
    input logic reset,
    input [31:0] \c[0] ,
    input logic en,
    input wire[2:0][1:0] op,
    input logic[WORD_WIDTH-1:0] a,
    input logic[WORD_WIDTH-1:0] b,
    output logic[WORD_WIDTH-1:0] y,
    output logic valid
);

endmodule