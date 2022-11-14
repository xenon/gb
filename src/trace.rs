use crate::cpu::Cpu;

pub fn run_trace(mut cpu: Cpu, max_cycles: Option<u64>, verbose: bool) {
    let mut cycles: u64 = 0;
    loop {
        if let Some(c) = max_cycles {
            if cycles >= c {
                break;
            }
        }
        let (pc, op, step_cycles) = cpu.step();
        if verbose {
            println!("({:#06x}): [{:#04x}]\t{}", pc, op, cpu.print());
        } else {
            println!("({:#06x}): [{:#04x}]", pc, op);
        }
        cycles += step_cycles as u64;
    }
    let (pc, op) = cpu.next_step();
    println!("({:#06x}): [{:#04x}]\t{}", pc, op, cpu.print());
}
