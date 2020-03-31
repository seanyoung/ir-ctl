use super::ast::*;
use super::irp;

use std::collections::HashMap;

// Here we parse an irp lang

#[derive(Debug)]
pub struct GeneralSpec {
    duty_cycle: Option<u8>,
    carrier: Option<i64>,
    lsb: bool,
    unit: f64,
}

pub struct Vartable {
    vars: HashMap<String, i64>,
}

impl Vartable {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: String, v: i64) {
        self.vars.insert(id, v);
    }

    pub fn get(&self, id: &str) -> Result<i64, String> {
        match self.vars.get(id) {
            Some(n) => Ok(*n),
            None => Err(format!("variable {} not defined", id)),
        }
    }
}

impl Expression {
    fn eval(&self, vars: &Vartable) -> Result<i64, String> {
        match self {
            Expression::Number(n) => Ok(*n),
            Expression::Identifier(id) => vars.get(id),
            Expression::Negative(e) => Ok(-e.eval(vars)?),
            Expression::Complement(e) => Ok(!e.eval(vars)?),
            Expression::Add(l, r) => Ok(l.eval(vars)? + r.eval(vars)?),
            Expression::Subtract(l, r) => Ok(l.eval(vars)? - r.eval(vars)?),
            Expression::Multiply(l, r) => Ok(l.eval(vars)? * r.eval(vars)?),
            Expression::Divide(l, r) => Ok(l.eval(vars)? / r.eval(vars)?),
            Expression::Modulo(l, r) => Ok(l.eval(vars)? % r.eval(vars)?),
            Expression::BitwiseAnd(l, r) => Ok(l.eval(vars)? & r.eval(vars)?),
            Expression::BitwiseOr(l, r) => Ok(l.eval(vars)? | r.eval(vars)?),
            Expression::BitwiseXor(l, r) => Ok(l.eval(vars)? ^ r.eval(vars)?),
            _ => unimplemented!(),
        }
    }
}

impl Unit {
    fn eval(&self, v: i64, spec: &GeneralSpec) -> Result<i64, String> {
        match self {
            Unit::Microseconds => Ok(v),
            Unit::Milliseconds => Ok(v * 1000),
            Unit::Pulses => match spec.carrier {
                Some(f) => Ok(v * 1000 / f),
                None => Err("pulses specified but no carrier given".to_string()),
            },
        }
    }
}

impl Duration {
    fn eval(&self, vars: &Vartable, spec: &GeneralSpec) -> Result<i64, String> {
        match self {
            Duration::FlashConstant(p, u) => u.eval((p * spec.unit) as i64, spec),
            Duration::GapConstant(p, u) => u.eval((-p * spec.unit) as i64, spec),
            Duration::FlashIdentifier(id, u) => {
                u.eval((vars.get(id)? as f64 * spec.unit) as i64, spec)
            }
            Duration::GapIdentifier(id, u) => {
                u.eval((-vars.get(id)? as f64 * spec.unit) as i64, spec)
            }
            _ => unimplemented!(),
        }
    }
}

pub fn render(input: &str, mut vars: Vartable) -> Result<Vec<u32>, String> {
    let parser = irp::protocolParser::new();

    let irp = parser.parse(input).map_err(|e| e.to_string())?;

    let gs = general_spec(&irp.general_spec)?;

    for i in irp.stream.stream {
        match i {
            IrStreamItem::Duration(d) => {
                d.eval(&vars, &gs);
            }
            IrStreamItem::Assignment(id, expr) => {
                vars.set(id, expr.eval(&vars)?);
            }
            _ => unimplemented!(),
        }
    }

    Err("not implemented".to_string())
}

fn general_spec(general_spec: &Vec<GeneralItem>) -> Result<GeneralSpec, String> {
    let mut res = GeneralSpec {
        duty_cycle: None,
        carrier: None,
        lsb: true,
        unit: 1.0,
    };

    let mut unit = None;
    let mut lsb = None;

    for i in general_spec {
        match i {
            GeneralItem::DutyCycle(d) => {
                if *d < 1.0 {
                    return Err("duty cycle less than 1% not valid".to_string());
                }
                if *d > 99.0 {
                    return Err("duty cycle larger than 99% not valid".to_string());
                }
                if res.duty_cycle.is_some() {
                    return Err("duty cycle specified twice".to_string());
                }

                res.duty_cycle = Some(*d as u8);
            }
            GeneralItem::Frequency(f) => {
                if res.carrier.is_some() {
                    return Err("carrier frequency specified twice".to_string());
                }

                res.carrier = Some((*f * 1000.0) as i64);
            }
            GeneralItem::OrderLsb | GeneralItem::OrderMsb => {
                if lsb.is_some() {
                    return Err("bit order (lsb,msb) specified twice".to_string());
                }

                lsb = Some(*i == GeneralItem::OrderLsb);
            }
            GeneralItem::Unit(p, u) => {
                if unit.is_some() {
                    return Err("unit specified twice".to_string());
                }

                unit = Some((p, u));
            }
        }
    }

    if let Some((p, u)) = unit {
        res.unit = match u {
            Unit::Pulses => {
                if let Some(f) = res.carrier {
                    p * 1000.0 / f as f64
                } else {
                    return Err("pulse unit specified without carrier frequency".to_string());
                }
            }
            Unit::Milliseconds => p * 1000.0,
            Unit::Microseconds => *p,
        }
    }

    if Some(false) == lsb {
        res.lsb = false;
    }
    Ok(res)
}

#[test]
fn test() {
    let mut vars = Vartable::new();

    vars.set("F".to_string(), 1);

    let res = render("{38.0k,564}<1,-1|1,-3>(16,-8,D:8,S:8,F:8,~F:8,1)", vars);

    assert_eq!(res, Ok(vec!(4500u32)));
}