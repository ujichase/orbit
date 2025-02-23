module  propertymodule ( input bit da , db , dclk, a, b, int count); 
  property  rc1;
    da |=> db ;
  endproperty
 
  baseP:assert  property ( @( posedge dclk )  rc1 );

ap_Count: assert property(@(posedge dclk) da |-> ##1 count==$past(count) +1);
endmodule
