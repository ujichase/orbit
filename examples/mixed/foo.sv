
module foo 
#(
    parameter int NUM = -9,
    parameter string WORD = "hello!"
) (
    input logic clk,
    input logic rst,
    input logic[4:0] a,
    input logic[4:0] b,
    input logic en,
    output logic[7:0] \result ,
    output logic valid
);

foo #(
  .NUM(NUM),
  .WORD(WORD)
) uX (
  .clk(clk),
  .rst(rst),
  .a(a),
  .b(b),
  .en(en),
  .\result (\result ),
  .valid(valid)
);

endmodule