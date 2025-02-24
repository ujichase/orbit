--------------------------------------------------------------------------------
-- Project: eel4712c.lab1
-- Author: Chase Ruskin
-- Course: Digital Design - EEL4712C
-- Creation Date: September 10, 2021
-- Entity: adder
-- Description:
--  Instantiates 6 full adders to create a 6-bit ripple carry adder
--  architecture.
--------------------------------------------------------------------------------

library ieee;
use ieee.std_logic_1164.all;
library work;
use work.const_pkg.all;

entity adder_mixed is
    generic (
        MY_WORD : string(1 to 3) := "FOO";
        WORD_WIDTH : positive := 6
    );
    port (
        INPUT1    : in  std_logic_vector(WORD_WIDTH-1 downto 0);
        input2    : in  std_logic_vector(WORD_WIDTH-1 downto 0);
        carry_in  : in  std_logic;
        sum       : out std_logic_vector(WORD_WIDTH-1 downto 0);
        carry_out : out std_logic
    );
end entity;

--defines a ripple-carry adder using 6 full adders
architecture struct of adder is

    --internal signal to propagate carry bit through each full adder
    signal carry_i : std_logic_vector(WORD_WIDTH downto 0) := (others => not FOO);
begin
    --first bit being carried in to adder
    carry_i(0) <= carry_in;

    --generate 6 full adder instances  
    ripple_carry : for ii in 0 to WORD_WIDTH-1 generate
        u_fa : work.lab1_pkg.fa
        port map(
            input1    => input1(ii),
            input2    => input2(ii),
            carry_in  => carry_i(ii),
            sum       => sum(ii),
            carry_out => carry_i(ii+1)
        );
    end generate ripple_carry;

    --last bit is to be carried out from adder
    carry_out <= carry_i(WORD_WIDTH);

end architecture;