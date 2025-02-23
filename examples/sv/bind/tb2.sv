// Reference: https://verificationacademy.com/forums/t/working-of-bind-construct/41111/2

//  Scenario2  ::  Without  bind
module  tb2;
 logic  ta , tb , tclk;

 designmodule  DM  ( .da(ta) , .db( tb ) , .dclk( tclk ) ) ;

 propertymodule  PM ( .pa( ta ) , .pb( tb ) , .pclk( tclk ) );

endmodule