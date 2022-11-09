use serde::Deserialize;

const CPU_TEST_PATH: &'static str =
    "C:\\Users\\user\\Documents\\Code\\rust-projects\\gameboy-test-data\\cpu_tests\\v1\\";

#[derive(Debug, Deserialize)]
pub struct CpuTest {
    name: String,
    #[serde(rename = "initial")]
    i: SystemState,
    #[serde(rename = "final")]
    f: SystemState,
    #[serde(rename = "cycles")]
    c: Vec<[String; 3]>,
}

#[derive(Debug, Deserialize)]
pub struct SystemState {
    cpu: CpuState,
    ram: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct CpuState {
    a: String,
    b: String,
    c: String,
    d: String,
    e: String,
    f: String,
    h: String,
    l: String,
    pc: String,
    sp: String,
}

#[test]
fn deserialize_test() {
    let json = r#"{
        "name": "00 00 ff",
        "initial": {
          "cpu": {
            "a": "0x00",
            "b": "0x01",
            "c": "0x02",
            "d": "0x03",
            "e": "0x03",
            "f": "0x00",
            "h": "0x04",
            "l": "0x05",
            "pc": "0x0100",
            "sp": "0xfffe"
          },
          "ram": [
            [
              "0x0100",
              "0x00"
            ]
          ]
        },
        "final": {
          "cpu": {
            "a": "0x00",
            "b": "0x01",
            "c": "0x02",
            "d": "0x03",
            "e": "0x03",
            "f": "0x00",
            "h": "0x04",
            "l": "0x05",
            "pc": "0x0101",
            "sp": "0xfffe"
          },
          "ram": [
            [
              "0x0100",
              "0x00"
            ]
          ]
        },
        "cycles": [
          [
            "0x0100",
            "0x00",
            "read"
          ]
        ]
      }
    "#;
    let test: std::result::Result<CpuTest, serde_json::Error> = serde_json::from_str(json);
    match test {
        Ok(test) => {
            println!("{:?}", test);
        }
        Err(e) => {
            eprintln!("Could not deserialize: {}", e);
            assert!(false);
        }
    }
}
