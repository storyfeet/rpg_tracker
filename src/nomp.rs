
use nom::IResult;
use nom::sequence::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::{map,peek,value};


use crate::proto_ex::ProtoX;
use crate::value::Value;


pub fn ident(s:&str)->IResult<&str,&str>{
    preceded(peek(alpha1),alphanumeric1)(s)
}

pub fn n_bool(s:&str)->IResult<&str,bool>{
    alt((
    value(true,alt((tag("true"),tag("TRUE"),tag("True")))),
    value(false,alt((tag("false"),tag("FALSE"),tag("False")))),
    ))(s)
}


pub fn dnd_value(s:&str)->IResult<&str,Value>{
    map(n_bool,|v|Value::Bool(v))(s)
            
}

#[cfg(test)]
mod nom_test{
    use super::*;
    #[test]
    fn test_nom_ident(){
        assert_eq!(ident("hello"),Ok(("","hello")));
        assert_eq!(ident("h2ello "),Ok((" ","h2ello")));
        assert!(ident("  hello").is_err());
        assert!(ident("1hello").is_err());
        assert!(ident(" abc").is_err());
        assert!(ident("123").is_err());
    }

    
    #[test]
    fn test_nom_value(){
        assert_eq!(dnd_value("true"),Ok(("",Value::Bool(true))));
        assert_eq!(dnd_value("True"),Ok(("",Value::Bool(true))));

    }

}



