use epxr::Expr;
use value::Value;

mod epxr;
mod parser;
mod utils;
mod value;

fn main() {
    const INPUT: &str = include_str!("./test.lay");
    let expr = Expr::parse(INPUT).unwrap();

    let id = expr.get_id("Foo").unwrap();
    let values = Value::prompt_for_values(&expr, id).unwrap();

    let data = values.format_value();
    println!("{}", utils::as_hex(&data));
}
