// Reference: https://verificationacademy.com/forums/t/working-of-bind-construct/41111/2

//  Scenario1  ::  Using  bind
module  tb1;

 // bind <target> <what to bind> [parameters] <instance of what to bind> [ports]
 bind  designmodule  propertymodule  dpM
  ( .da(da), .db(db),.dclk(dclk), .a(a), .b(b), .count(count));
 
endmodule


// TODO: There is more to bind... see https://vlsi.pro/sva-basics-bind/ for
// more ways bind can be used. Only the above example (`tb1`) is currently supported.