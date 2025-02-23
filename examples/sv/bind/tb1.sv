// Reference: https://verificationacademy.com/forums/t/working-of-bind-construct/41111/2

//  Scenario1  ::  Using  bind
module  tb1;

 // bind <target> <what to bind> [parameters] <instance of what to bind> [ports]
 bind  designmodule  propertymodule  dpM
  ( .da(da), .db(db),.dclk(dclk), .a(a), .b(b), .count(count));
 
endmodule
