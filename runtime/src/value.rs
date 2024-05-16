#[derive(PartialEq, Debug, Clone)]
pub enum ValueType {
    ValBool,
    ValNil,
    ValNumber,
}

#[derive(Clone, Copy)]
pub union ValuePayload {
    boolean: Boolean,
    number: Number,
}

#[derive(Clone)]
pub struct Value {
    value_type: ValueType,
    as_union: ValuePayload,
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_type {
            ValueType::ValBool => {
                write!(f, "Value {{ {:?}: {} }}", self.value_type, self.as_bool())
            }
            ValueType::ValNil => write!(f, "Value {{ {:?} }}", self.value_type),
            ValueType::ValNumber => {
                write!(f, "Value {{ {:?}: {} }}", self.value_type, self.as_number())
            }
        }
    }
}

pub type Boolean = bool;
pub type Number = f64;

impl Value {
    pub fn from_bool(value: Boolean) -> Self {
        Self {
            value_type: ValueType::ValBool,
            as_union: ValuePayload { boolean: value },
        }
    }

    pub fn from_nil() -> Self {
        Self {
            value_type: ValueType::ValNil,
            as_union: ValuePayload { number: 0.0 },
        }
    }

    pub fn from_number(value: Number) -> Self {
        Self {
            value_type: ValueType::ValNumber,
            as_union: ValuePayload { number: value },
        }
    }

    pub fn as_bool(&self) -> Boolean {
        return unsafe { self.as_union.boolean };
    }

    pub fn as_number(&self) -> Number {
        return unsafe { self.as_union.number };
    }

    pub fn is_bool(&self) -> bool {
        return self.value_type == ValueType::ValBool;
    }

    pub fn is_nil(&self) -> bool {
        return self.value_type == ValueType::ValNil;
    }

    pub fn is_number(&self) -> bool {
        return self.value_type == ValueType::ValNumber;
    }

    pub fn get_type(&self) -> &ValueType {
        return &self.value_type;
    }

    pub fn print(&self) {
        match self.value_type {
            ValueType::ValBool => {
                if self.as_bool() {
                    print!("true");
                } else {
                    print!("false");
                }
            }
            ValueType::ValNil => {
                print!("nil")
            }
            ValueType::ValNumber => {
                print!("{}", self.as_number());
            }
        }
    }
}
