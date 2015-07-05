// This is not a module, but meant to be included with `include!`
// The culprit: apparently I can't normally span impls across many files

impl Context {
    fn fun_name(&self) -> &str {
        &self.functions.table[self.current_fun_pos].1
    }

    pub fn run(&mut self) -> (Vec<Literal>, VmError) {
        let mut result = Ok(());

        if self.program.len() == 0 {
            debug!("There is no code");
            return (vec![], ThereIsNoCode);
        }

        self.current_fun = self.functions.table[self.current_fun_pos].0;

        debug!("Program: {:?}", self.program);
        debug!("Starting at {:?}", self.fun_name());

        while let Ok(()) = result {
            result = if self.counter > self.current_fun.end_pos {
                debug!("Implicit return at end of {}", self.fun_name());
                self.ret()
            } else {
                debug!("{}\tPC {:?} {:?}\tStack: {:?}",
                         self.fun_name(),
                         self.counter,
                         self.program[self.counter],
                         self.data_stack);

                self.step()
            };
        }

        let error = match result {
            Err(a) => a,
            Ok(()) => unreachable!(),
        };

        debug!("-");
        debug!("{}\tPC {:?} Stack {:?} Exit: {:?}",
                 self.fun_name(),
                 self.counter,
                 self.data_stack, error);

        (self.data_stack.clone(), error)
    }

    fn step(&mut self) -> VmResult<()> {
        let instr = self.program[self.counter];
        self.counter += 1;

        match instr {
            Push(val) => Ok(self.data_stack.push(val)),
            Pop => self.pop().map(|_| ()),

            Dup => self.dup(),
            Swap(stack_pos) => self.swap(stack_pos),

            BinOp(op) => self.binop(op),
            Branch(cond, offset) => self.branch(cond, offset),
            Jump(offset) => self.jump(offset),

            Call(function_pos) => self.call(function_pos),
            Return => self.ret(),
        }
    }

    fn get_fun(&self, function_pos: ProgramPos) -> VmResult<(Function, usize)> {
        let (f, p, _) = try!(self.functions
                          .search_start_pos(function_pos)
                          .ok_or(InvalidFunctionPosition));

        Ok((f, p))
    }

    fn call(&mut self, function_pos: ProgramPos) -> VmResult<()> {
        // the stack was previously advanced
        self.call_stack.push(CallFrame {
            return_pos: self.counter - 1,
            function_start: self.current_fun.start_pos
        });
        self.counter = function_pos;
        let (fun, pos) = try!(self.get_fun(function_pos));
        self.current_fun = fun;
        self.current_fun_pos = pos;

        Ok(())
    }

    fn ret(&mut self) -> VmResult<()> {
        let frame = try!(self.call_stack.pop().ok_or(Halted));
        self.counter = frame.return_pos + 1;
        let (fun, pos) = try!(self.get_fun(frame.function_start));
        self.current_fun = fun;
        self.current_fun_pos = pos;

        Ok(())
    }

    fn swap(&mut self, pos: StackPos) -> VmResult<()> {
        let last = self.data_stack.len() - 1;

        if pos > last {
            return Err(InvalidStackPosition);
        }

        self.data_stack.swap(last, last - pos);
        Ok(())
    }

    fn top(&self) -> VmResult<Literal> {
        self.data_stack.last().map(|v| *v).ok_or(StackUnderflow)
    }

    fn pop(&mut self) -> VmResult<Literal> {
        self.data_stack.pop().ok_or(StackUnderflow)
    }

    fn dup(&mut self) -> VmResult<()> {
        let top = try!(self.top());
        self.data_stack.push(top);
        Ok(())
    }

    fn binop(&mut self, op: Operation) -> VmResult<()> {
        let a = try!(self.pop());
        let b = try!(self.pop());

        let v = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            Mod => a % b,
        };

        self.data_stack.push(v);
        Ok(())
    }

    fn branch(&mut self, cond: BranchCondition, offset: JumpOffset) -> VmResult<()> {
        let a = try!(self.top());

        match cond {
            Equal => if a == 0 { try!(self.jump(offset)); },
            NotEqual => if a != 0 { try!(self.jump(offset)); },
            GreaterThan => if a > 0 { try!(self.jump(offset)); },
            LessThan => if a < 0 { try!(self.jump(offset)); },
            GreaterEqual => if a >= 0 { try!(self.jump(offset)); },
            LessEqual => if a <= 0 { try!(self.jump(offset)); },
        }
        Ok(())
    }

    fn jump(&mut self, offset: JumpOffset) -> VmResult<()> {
        let address = ((self.counter as JumpOffset) + offset - 1) as StackPos;

        if address < self.current_fun.start_pos {
            return Err(JumpOffsetTooSmall);
        }

        // I'll let jumping to the position immediately after the program.
        // This means "return". Perhaps I should remove the Return instruction.
        if address - 1 > self.current_fun.end_pos {
            return Err(JumpOffsetTooLarge);
        }

        self.counter = address;
        Ok(())
    }
}
