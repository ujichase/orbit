
library ieee;
use ieee.std_logic_1164.all;

library work;
use work.a.all;

library work;
use work.b_pkg.all;

entity b is
    port (
        pulse: out std_logic
    );
end entity;


architecture rtl of b is

begin

    pulse <= '1';

end architecture;