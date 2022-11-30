use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::gb::cpu::registers::{Reg8, Registers};

const ALU_TEST_PATH: &'static str =
    "C:\\Users\\user\\Documents\\Code\\rust-projects\\gameboy-test-data\\alu_tests\\v1\\";

#[derive(Deserialize)]
pub struct AluTest {
    x: String,
    y: String,
    flags: String,
    result: Result,
}

#[derive(Deserialize)]
pub struct Result {
    value: String,
    flags: String,
}

fn reader(path: String) -> Option<BufReader<File>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        Some(reader)
    } else {
        None
    }
}

fn to_hex(test: &AluTest) -> (u8, u8, u8, u8, u8) {
    let x = u8::from_str_radix(&test.x[2..], 16).unwrap();
    let y = u8::from_str_radix(&test.y[2..], 16).unwrap();
    let flags = u8::from_str_radix(&test.flags[2..], 16).unwrap();
    let res = u8::from_str_radix(&test.result.value[2..], 16).unwrap();
    let res_flags = u8::from_str_radix(&test.result.flags[2..], 16).unwrap();
    (x, y, flags, res, res_flags)
}

#[test]
fn adc() {
    let reader = reader(format!("{}adc.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.adc(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn add() {
    let reader = reader(format!("{}add.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.add(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn and() {
    let reader = reader(format!("{}and.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.and(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn bit() {
    let reader = reader(format!("{}bit.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        regs.bit(x, y);

        let val_flags = regs.get_8(Reg8::F);
        if val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - flags [{}] expected flags [{}]",
                test_num, val_flags, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn ccf() {
    let reader = reader(format!("{}ccf.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        regs.ccf();
        let val_flags = regs.get_8(Reg8::F);

        if val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - flags [{}] expected flags [{}]",
                test_num, val_flags, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn cp() {
    let reader = reader(format!("{}cp.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        regs.cp(y);
        let val_flags = regs.get_8(Reg8::F);

        if val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - flags [{}] expected flags [{}]",
                test_num, val_flags, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn cpl() {
    let reader = reader(format!("{}cpl.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.cpl();
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn daa() {
    let reader = reader(format!("{}daa.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.daa();
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn or() {
    let reader = reader(format!("{}or.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.or(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn res() {
    let reader = reader(format!("{}res.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.res(x, y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn rl() {
    let reader = reader(format!("{}rl.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.rl(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn rlc() {
    let reader = reader(format!("{}rlc.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.rlc(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn rr() {
    let reader = reader(format!("{}rr.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.rr(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn rrc() {
    let reader = reader(format!("{}rrc.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.rrc(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn sbc() {
    let reader = reader(format!("{}sbc.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.sbc(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn scf() {
    let reader = reader(format!("{}scf.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        regs.scf();
        let val_flags = regs.get_8(Reg8::F);

        if val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - flags [{}] expected flags [{}]",
                test_num, val_flags, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn set() {
    let reader = reader(format!("{}set.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.set(x, y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn sla() {
    let reader = reader(format!("{}sla.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.sla(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn sra() {
    let reader = reader(format!("{}sra.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.sra(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn srl() {
    let reader = reader(format!("{}srl.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.srl(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn sub() {
    let reader = reader(format!("{}sub.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.sub(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn swap() {
    let reader = reader(format!("{}swap.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::F, flags);
        let val = regs.swap(x);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}

#[test]
fn xor() {
    let reader = reader(format!("{}xor.json", ALU_TEST_PATH));
    assert!(reader.is_some());
    eprintln!("File reader created!");
    let tests: std::result::Result<Vec<AluTest>, serde_json::Error> =
        serde_json::from_reader(reader.unwrap());
    if let Err(e) = &tests {
        eprintln!("Error: {}", e);
        assert!(false);
    }
    eprintln!("File read successfully!");
    let mut count = 0;
    let mut fail_count = 0;

    let mut regs = Registers::new();

    for (test_num, test) in tests.unwrap().iter().enumerate() {
        let (x, y, flags, res, res_flags) = to_hex(test);
        regs.set_8(Reg8::A, x);
        regs.set_8(Reg8::F, flags);
        let val = regs.xor(y);
        let val_flags = regs.get_8(Reg8::F);

        if res != val || val_flags != res_flags {
            fail_count += 1;
            eprintln!(
                "[fail    {}] - res [{}], flags [{}] expected res [{}], flags [{}]",
                test_num, val, val_flags, res, res_flags
            );
        }
        count += 1;
    }
    eprintln!("{} tests ran, {} failed", count, fail_count);
    assert_eq!(fail_count, 0);
}
