use super::algebra;
use super::error::{Result,Error};

#[derive(Debug, Clone)]
pub enum List {
    Algebra(algebra::Value),
    List(Vec<List>),
}

impl List {
    pub fn new(sizes : &[usize]) -> Self {
        if sizes.len() == 0{
            List::Algebra(algebra::Value::default())
        } else {
            let mut l = Vec::new();
            for _ in 0..sizes[0] {
                l.push(List::new(&sizes[1..]));
            }
            List::List(l)
        }
    }

    pub fn get(&self, indexes : &[usize]) -> Result<&List> {
        if indexes.len() > 0 {
            match self {
                List::Algebra(_) =>
                    Err(Error::InvalidSelector(format!("index at [{}] contains a value",indexes[0]))),
                List::List(v) => {
                    if indexes[0] >= v.len() {
                        Err(Error::InvalidSelector(format!("index at [{}] too large",indexes[0])))
                    } else {
                        v[indexes[0]].get(&indexes[1..])
                    }
                }
            }
        } else {
            Ok(self)    
        }
    }

    pub fn set(&mut self, value: &algebra::Value, indexes : &[usize]) -> Result<()> {
        match self {
            List::Algebra(_) =>
                Err(Error::InvalidSelector(format!("index at [{}] contains a value",indexes[0]))),
            
            List::List(v) => {
                if indexes.len()==0 || indexes[0] >= v.len() {
                    Err(Error::InvalidSelector(format!("invalid index for {:?}",v)))
                } else if indexes.len() == 1 {
                    v[indexes[0]] = List::Algebra(value.clone());
                    Ok(())
                } else {
                    v[indexes[0]].set(value, &indexes[1..])  
                }
            }
        }
    }

}


