module  designmodule  (input bit da, dclk, output bit db);
  always @( posedge dclk )  db <= da;
  bit a, b; // used internally in the desing and are not IO
  int count; 
  always @( posedge dclk) if(da) count <= count+1; 
endmodule
