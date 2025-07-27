use tokio;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestStruct {
    name: String,
    value: i32,
}

#[tokio::main]
async fn main() {
    println!("hello from rust test project");
    
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    
    println!("test struct: {} = {}", test.name, test.value);
}
