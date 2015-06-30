// This is not a module, but meant to be included with `include!`
// The culprit: apparently I can't normally span impls across many files

impl Context {
    pub fn link(&mut self, new_functions: Vec<(&str, Vec<Opcode<&str>>)>) -> LinkResult<()> {
        self.extend_function_table(&new_functions);

        for (_, code) in new_functions {
            try!(self.link_step(code));
        }
        Ok(())
    }

    fn extend_function_table(&mut self, new_functions: &[(&str, Vec<Opcode<&str>>)]) {
        let mut start_pos = self.program.len() as ProgramPos;

        for &(name, ref code) in new_functions {
            let end_pos = start_pos + (code.len() as ProgramPos) - 1;

            self.functions.table.push((Function {
                start_pos: start_pos,
                end_pos: end_pos
            }, name.to_owned()));

            start_pos = end_pos + 1;
        }
    }

    fn link_step(&mut self, code: Vec<Opcode<&str>>) -> LinkResult<()> {
        for v in code {
            let instr : Opcode<ProgramPos> = match v {
                Call(name) => {
                    let (function, _) = try!(self.functions
                                        .search_name(name)
                                        .ok_or(FunctionNameNotFound));

                    Call(function.start_pos)
                },

                // do not repeat yourself! this is intolerable..
                
                Push(lit) => Push(lit),
                Pop => Pop,
                Swap(pos) => Swap(pos),
                BinOp(op) => BinOp(op),
                Branch(cond, offset) => Branch(cond, offset),
                Jump(offset) => Jump(offset),
                Return => Return,
            };

            self.program.push(instr);
        }
        Ok(())
    }
}
