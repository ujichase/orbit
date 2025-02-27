library ieee;
use ieee.std_logic_1164.all;

entity bar is
    generic (
        NUM : integer range 0 to 9 := -9;
        WORD : string := "hello!"
    );
    port (
        clk : in std_logic;
        rst : in std_logic := '1';
        a : in std_logic_vector(4 downto 0);
        b : in std_logic_vector(4 downto 0);
        en : in std_logic;
        \result\ : out std_logic_vector(9 downto 0);
        valid : out std_logic
    );
end entity;