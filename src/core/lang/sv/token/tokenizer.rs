//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use std::str::FromStr;

use crate::core::lang::{
    lexer::{self, Token, TokenError, Tokenize, TrainCar},
    sv::error::SystemVerilogError,
    verilog::token::{token::VerilogToken, tokenizer::char_set},
};

use super::token::SystemVerilogToken;

#[derive(Debug, PartialEq)]
struct SystemVerilogElement(
    Result<lexer::Token<SystemVerilogToken>, lexer::TokenError<SystemVerilogError>>,
);

#[derive(PartialEq)]
pub struct SystemVerilogTokenizer {
    tokens: Vec<SystemVerilogElement>,
}

impl Tokenize for SystemVerilogTokenizer {
    type TokenType = SystemVerilogToken;
    type Err = SystemVerilogError;

    fn tokenize(s: &str) -> Vec<Result<lexer::Token<Self::TokenType>, lexer::TokenError<Self::Err>>>
    where
        <Self as Tokenize>::Err: std::fmt::Display,
    {
        let mut train = TrainCar::new(s.chars());
        // store results here as we consume the characters
        let mut tokens: Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = train.consume() {
            // skip over whitespace
            if char_set::is_whitespace(&c) == true {
                continue;
            }
            let tk_loc = train.locate().clone();
            // peek at next character
            let next = train.peek();
            // add a token to the list
            tokens.push(
                if char_set::is_letter(&c) == true || char_set::UNDER_SCORE == c {
                    // collect keyword or identifier
                    match SystemVerilogToken::consume_word(&mut train, c) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::ESC == c {
                    // collect identifier (escaped)
                    match VerilogToken::consume_escaped_identifier(&mut train) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::DOUBLE_QUOTE == c {
                    // collect a string literal
                    match VerilogToken::consume_str_literal(&mut train) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::is_digit(&c) == true
                    || char_set::SINGLE_QUOTE == c
                    || ((char_set::PLUS == c || char_set::MINUS == c)
                        && next.is_some_and(|d| char_set::is_digit(&d) == true))
                {
                    // collect a number
                    match VerilogToken::consume_number(&mut train, c) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::FWD_SLASH == c
                    && next.is_some_and(|d| d == &char_set::FWD_SLASH)
                {
                    // collect single-line comment
                    match VerilogToken::consume_oneline_comment(&mut train) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::FWD_SLASH == c && next.is_some_and(|f| f == &char_set::STAR) {
                    // collect block comment
                    match VerilogToken::consume_block_comment(&mut train) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else if char_set::DOLLAR_SIGN == c {
                    // collect system task/function identifier
                    match SystemVerilogToken::consume_word(&mut train, c) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                    // todo!("collect system task")
                } else if char_set::GRAVE_ACCENT == c {
                    // collect compiler directive
                    match VerilogToken::consume_compiler_directive(&mut train) {
                        Ok(tk) => Ok(Token::new(SystemVerilogToken::from(tk), tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                } else {
                    // collect operator/delimiter
                    match SystemVerilogToken::consume_operator(&mut train, Some(c)) {
                        Ok(tk) => Ok(Token::new(tk, tk_loc)),
                        Err(e) => Err(TokenError::new(e, train.locate().clone())),
                    }
                },
            );
        }
        // push final EOF token
        let mut tk_loc = train.locate().clone();
        tk_loc.next_col();
        tokens.push(Ok(Token::new(SystemVerilogToken::EOF, tk_loc)));
        tokens
    }
}

impl FromStr for SystemVerilogTokenizer {
    type Err = SystemVerilogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_source_code(s))
    }
}

impl SystemVerilogTokenizer {
    /// Creates a new `SystemVerilogTokenizer` struct.
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Generates a `SystemVerilogTokenizer` struct from source code `s`.
    ///
    /// @TODO If `skip_err` is true, it will silently omit erroneous parsing from the
    /// final vector and guarantee to be `Ok`.
    pub fn from_source_code(s: &str) -> Self {
        Self {
            tokens: Self::tokenize(s)
                .into_iter()
                .map(|f| SystemVerilogElement(f))
                .collect(),
        }
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    ///
    /// This `fn` also filters out `Comment`s. To include `Comment` tokens, see
    /// `into_tokens_all`.
    pub fn into_tokens(self) -> Vec<lexer::Token<SystemVerilogToken>> {
        self.tokens
            .into_iter()
            .filter_map(|f| match f.0 {
                Ok(t) => match t.as_ref() {
                    SystemVerilogToken::Comment(_) => None,
                    _ => Some(t),
                },
                Err(_) => None,
            })
            .collect()
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn into_tokens_all(self) -> Vec<lexer::Token<SystemVerilogToken>> {
        self.tokens
            .into_iter()
            .filter_map(|f| match f.0 {
                Ok(t) => Some(t),
                Err(_) => None,
            })
            .collect()
    }

    /// Transforms the list of results into a list of tokens, silently skipping over
    /// errors.
    pub fn as_tokens_all(&self) -> Vec<&lexer::Token<SystemVerilogToken>> {
        self.tokens
            .iter()
            .filter_map(|f| match &f.0 {
                Ok(t) => Some(t),
                Err(_) => None,
            })
            .collect()
    }
}

impl std::fmt::Debug for SystemVerilogTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.tokens {
            write!(
                f,
                "{}\t{:?}\n",
                tk.0.as_ref().unwrap().locate(),
                tk.0.as_ref().unwrap()
            )?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lang::sv::token::operator::Operator;

    #[test]
    fn ut_valid_operators() {
        use Operator::*;

        let tests = vec![
            "<<<=", "<->", ">>>=", "<<", ">>>", "!=?", "?:", "?", "++", "&=", "::", ":", "/=",
            "->", "==?", ".*",
        ];
        let answers = vec![
            TripleShiftAssignL,
            DoubleArrow,
            TripleShiftAssignR,
            LogicShiftL,
            ArithShiftR,
            NotEqQuestion,
            QuestionColon,
            Question,
            DoublePlus,
            AndAssign,
            ScopeResolution,
            Colon,
            DivAssign,
            ArrowR,
            DoubleEqQuestion,
            DotStar,
        ];
        for i in 0..tests.len() {
            println!("{}", tests[i]);

            let ops: Vec<Operator> = SystemVerilogTokenizer::tokenize(tests[i])
                .into_iter()
                .filter(|f| {
                    f.is_err() == true
                        || (f.is_ok() && f.as_ref().unwrap().as_ref().is_eof() == false)
                })
                .map(|f| {
                    println!("{:?}", f);
                    f.unwrap().take().takes_delimiter().take().unwrap()
                })
                .collect();

            assert_eq!(ops.len(), 1);
            assert_eq!(ops[0], answers[i]);
        }
    }

    #[test]
    fn ut_tokenizer_stress() {
        let s = r#"// University of Florida
// Lab 3 EEL6935
// This file contains a top-level module timing example that you are required
// to optimize to as fast a frequency as possible. Make sure to use the
// provided timing_example_tb.sv to ensure that any changes are still
// functionally correct.
//
// Unless the comments say otherwise, you are allowed to make any change you
// want with the exception of removing modules.
//
// You are allowed to sacrifice performance to improve the clock frequency, up
// to a limit of 5% performance overhead, which is measured by the testbench.


module bit_diff
  #(
    parameter WIDTH
    )
   (
    input logic                                 clk,
    input logic                                 rst,
    input logic                                 go,
    input logic [WIDTH-1:0]                     data, 
    output logic signed [$clog2(2*WIDTH+1)-1:0] result,
    output logic                                done    
    );
   
   typedef enum                                 {START, COMPUTE, RESTART} state_t;
   state_t state_r;

   logic [$bits(data)-1:0]                      data_r;
   logic [$bits(result)-1:0]                    result_r;
   logic [$clog2(WIDTH)-1:0]                    count_r;
   logic signed [$clog2(2*WIDTH+1)-1:0]         diff_r;
   logic                                        done_r;
   
   assign result = result_r;
   assign done = done_r;

   always @(posedge clk or posedge rst) begin
      if (rst == 1'b1) begin     
         
         result_r <= '0;
         done_r <= 1'b0;                 
         diff_r <= '0;   
         count_r <= '0;
         data_r <= '0;   
         state_r <= START;       
      end
      else begin         
         case (state_r)
           START : begin   
              if (go == 1'b1) begin 
                 done_r <= 1'b0;
                 result_r <= '0;                 
                 diff_r <= '0;
                 count_r <= '0;
                 data_r <= data;
                 state_r <= COMPUTE;
              end
           end

           COMPUTE : begin
              logic [$bits(diff_r)-1:0] next_diff;
              next_diff = data_r[0] == 1'b1 ? diff_r + 1'b1 : diff_r - 1'b1;
              diff_r <= next_diff;            
              data_r <= data_r >> 1;
              count_r <= count_r + 1'b1;

              if (count_r == WIDTH-1) begin
                 result_r <= next_diff;
                 done_r <= 1'b1;
                 state_r <= RESTART;
              end
           end
        
           RESTART : begin
              if (go == 1'b1) begin              
                 count_r <= '0;
                 data_r <= data;
                 diff_r <= '0;
                 done_r <= 1'b0;                 
                 state_r <= COMPUTE;
              end
           end
         endcase          
      end      
   end   
endmodule


module fifo
  #(
    parameter WIDTH,
    parameter DEPTH,
    parameter int ALMOST_FULL_THRESHOLD=DEPTH-1
    )
   (
    input logic              clk,
    input logic              rst,
    output logic             full,
    output logic             almost_full,
    input logic              wr_en,
    input logic [WIDTH-1:0]  wr_data,
    output logic             empty, 
    input logic              rd_en,
    output logic [WIDTH-1:0] rd_data  
    );
  
   logic [WIDTH-1:0]         ram[DEPTH];
   logic [$clog2(DEPTH)-1:0] wr_addr_r, rd_addr_r;

   localparam int            COUNT_WIDTH = $clog2(DEPTH)+1;   
   logic [COUNT_WIDTH-1:0]   count_r;
   logic                     valid_wr, valid_rd;

   // optimization #4: remove unncessary combinational logic for computing rd_addr
   // @note: add +1 to the pipeline latency

   always @(posedge clk) begin
      if (valid_wr == 1'b1) begin 
         ram[wr_addr_r] = wr_data; 
      end
      rd_data <= ram[rd_addr_r];
   end

   logic[COUNT_WIDTH-1:0] count_operand, count_next;
   
   always @(posedge clk or posedge rst) begin
      if (rst == 1'b1) begin
         rd_addr_r <= '0;
         wr_addr_r <= '0;
         count_r <= '0;  
      end
      else begin
         count_r <= count_next;

         if (valid_wr == 1'b1) begin
            wr_addr_r <= wr_addr_r + 1'b1;         
         end
         if (valid_rd == 1'b1) begin 
            // @note: the rd_addr is incremented on valid_rd
            rd_addr_r <= rd_addr_r + 1'b1;       
         end
      end
   end 

   // optimization #2: explicitly force synthesis tool into particular architecture
   // for implementing a multiplexer for modifying the count register
   always_comb begin
      case ({valid_wr, valid_rd})
         // +1 when writing
         2'b10   : count_operand = WIDTH'(1);
         // -1 when reading
         2'b01   : count_operand = '1;  // equivalent to -1 (2's complement)
         // net zero change
         default : count_operand = '0;	
      endcase

      count_next = count_r + count_operand;
   end
   
   assign valid_wr = wr_en && !full;
   assign valid_rd = rd_en && !empty;
   assign almost_full = count_r == ALMOST_FULL_THRESHOLD;   
   assign full = count_r == DEPTH;
   assign empty = count_r == 0;
    
endmodule


module timing_example
  #(
    // DO NOT CHANGE ANY OF THESE DEFAULTS
    parameter int INPUT_WIDTH=32,
    parameter int OUTPUT_WIDTH=8,
    parameter int COUNTER_WIDTH=64,
    parameter int NUM_PIPELINES=16
    )
   (
    input logic                      clk,
    input logic                      rst,
    input logic [INPUT_WIDTH-1:0]    data_in,
    input logic                      data_in_valid,
    input logic [OUTPUT_WIDTH-1:0]   pipe_in[NUM_PIPELINES],
    output logic                     ready,
    output logic [OUTPUT_WIDTH-1:0]  data_out,
    output logic                     data_out_valid, 
    output logic [COUNTER_WIDTH-1:0] count
    );

   logic [$clog2(2*INPUT_WIDTH+1)-1:0] bit_diff_out;
   logic                               bit_diff_done, bit_diff_done_r;
   logic                               first_execution_r;   
   
   // optimization #7: change FIFO parameters (from 512 to 4)
   localparam int                      FIFO_DEPTH = 4;
   logic [$bits(bit_diff_out)-1:0]     fifo_rd_data;
   logic                               fifo_wr_en, fifo_rd_en;
   logic                               fifo_full, fifo_almost_full, fifo_empty;   

   // DO NOT CHANGE THE WIDTH ANY THIS SIGNAL
   logic [63:0]                      total_count_r;
      
   // Perform a bit_diff on data_in.
   bit_diff #(
      .WIDTH(INPUT_WIDTH)
   ) bit_diff_ (
      .go(data_in_valid), 
      .data(data_in),
      .result(bit_diff_out),
      .done(bit_diff_done), 
      .*
   );

   logic bit_diff_done_buf;
   logic [$clog2(2*INPUT_WIDTH+1)-1:0] bit_diff_out_buf;

   // optimization #8: register the output of the bit_diff module
   always_ff @(posedge clk) begin
      bit_diff_out_buf <= bit_diff_out;
   end
    
   // Count the total number of bit_diff executions since reset
   always_ff @(posedge clk or posedge rst) begin
      if (rst == 1'b1) begin
         total_count_r <= '0;
         bit_diff_done_r <= 1'b0;
         fifo_wr_en <= 1'b0;         
         first_execution_r <= 1'b1;      
      end
      else begin
         fifo_wr_en <= 1'b0;        
         
         if (data_in_valid) begin first_execution_r <= 1'b0; end
         
         bit_diff_done_r <= bit_diff_done;
         
         // If the current bit_diff_done is asserted and the previous cycle
         // wasn't, we just had a new completion, so increment the count.
         if (bit_diff_done && !bit_diff_done_r) begin
            total_count_r <= total_count_r + 1'b1;

            // Write the output to the FIFO upon completion.
            fifo_wr_en <= 1'b1;     
         end
      end
   end

   // optimization #3: enable a multicycle path for total_count_r due to very long
   // carry-chain adder
   assign count = total_count_r;   
   assign ready = first_execution_r || (bit_diff_done_r && !fifo_almost_full);
         
   fifo #(
      .WIDTH($bits(bit_diff_out)), 
      .DEPTH(FIFO_DEPTH)
   ) fifo_ (
      .wr_en(fifo_wr_en), 
      .full(fifo_full), 
      .almost_full(fifo_almost_full), 
      .wr_data(bit_diff_out_buf), 
      .rd_en(fifo_rd_en), 
      .rd_data(fifo_rd_data), 
      .empty(fifo_empty), 
      .*
   );

   logic [OUTPUT_WIDTH-1:0]     fifo_rd_data_buf;

   // optimization #1: add a buffer between the multiply and fifo output
   // @note: adjust the pipeline latency by +1 cycle
   always_ff @(posedge clk) begin
      fifo_rd_data_buf <= 8'(fifo_rd_data);
   end

   assign fifo_rd_en = !fifo_empty;

   // optimization #5: reduce fanout using multi-level register duplication
   localparam int FANOUT_LVL_1 = NUM_PIPELINES/4;

   (* dont_merge *) logic [OUTPUT_WIDTH-1:0] 	 pipe_in_l_lvl1[FANOUT_LVL_1];
   (* dont_merge *) logic [OUTPUT_WIDTH-1:0] 	 pipe_in_l_lvl2[NUM_PIPELINES];
       
   always_ff @(posedge clk or posedge rst) begin
      if (rst == 1'b1) begin
         for (int i=0; i < FANOUT_LVL_1; i++) pipe_in_l_lvl1[i] <= '0;
         for (int i=0; i < NUM_PIPELINES; i++) pipe_in_l_lvl2[i] <= '0;	 	 
      end
      else begin     
         // Manually create the register duplication hierarchy.
         for (int i=0; i < FANOUT_LVL_1; i++) pipe_in_l_lvl1[i] <= fifo_rd_data_buf;
         for (int i=0; i < NUM_PIPELINES; i++) pipe_in_l_lvl2[i] <= pipe_in_l_lvl1[i/4];
      end
   end
      
   ////////////////////////////////////////////////////////
   // Instantiate a multiply-add tree.
   
   logic [OUTPUT_WIDTH-1:0] pipe_in_r[NUM_PIPELINES], mult_out[NUM_PIPELINES], add_l0[8], add_l1[4], add_l2[2];  
   
   always_ff @(posedge clk or posedge rst) begin
      // @note: reset reduction does not improve timing
      if (rst == 1'b1) begin
         for (int i=0; i < NUM_PIPELINES; i++) begin
            pipe_in_r[i] <= '0;
            mult_out[i] <= '0;
         end
      end else begin                  
         for (int i=0; i < NUM_PIPELINES; i++) begin
            // Register all the pipeline inputs. You can assume these inputs 
            // never change in the middle of execution.
            pipe_in_r[i] <= pipe_in[i];     
            mult_out[i] <= 8'(pipe_in_l_lvl2[i]) * pipe_in_r[i];
         end         
      end
   end
   
   // optimization #6: place flip-flops through adder tree
   always_ff @(posedge clk) begin
      for (int i=0; i < 8; i++) add_l0[i] <= mult_out[2*i] + mult_out[2*i+1];
      for (int i=0; i < 4; i++) add_l1[i] <= add_l0[2*i] + add_l0[2*i+1];
      for (int i=0; i < 2; i++) add_l2[i] <= add_l1[2*i] + add_l1[2*i+1];
      data_out <= add_l2[0] + add_l2[1];
   end

   ////////////////////////////////////////////////////
   // Logic for valid_out

   // IF YOU MAKE CHANGES THAT INCREASE LATENCY OF THE MULTIPLY-ADD TREE, YOU
   // WILL NEED TO CHANGE THIS LOCALPARAM.
   localparam int                        PIPE_LATENCY = 1+1+4+2;
   logic [0:PIPE_LATENCY-1]              valid_delay_r;
   
   always_ff @(posedge clk or posedge rst) begin
      if (rst == 1'b1) begin
         for (int i=0; i < PIPE_LATENCY; i++) valid_delay_r[i] = '0;
      end
      else begin
         valid_delay_r[0] <= fifo_rd_en;       
         for (int i=1; i < PIPE_LATENCY; i++) valid_delay_r[i] <= valid_delay_r[i-1];    
      end      
   end

   assign data_out_valid = valid_delay_r[PIPE_LATENCY-1];
   
endmodule
        "#;

        let tokens = SystemVerilogTokenizer::tokenize(s);
        assert_eq!(
            tokens
                .iter()
                .find(|f| {
                    if let Some(e) = f.as_ref().err() {
                        println!("{:?}", e);
                        true
                    } else {
                        false
                    }
                })
                .is_none(),
            true
        );
    }
}
