use crate::token::Token;
use crate::expr::Expr;
use crate::error::{LineError,ActionError};
use crate::token::TokPrev;
use crate::prev_iter::{Backer,LineCounter};
use crate::value::Value;
use crate::scope::Scope;
use crate::proto::Proto;
use crate::api_funcs;

#[derive(PartialEq, Clone, Debug)]
pub struct ProtoX{
    d:i32,
    dot:bool,
    var:bool,
    exs:Vec<Expr>,
    params:Option<Vec<Expr>>,
}

impl ProtoX{
    pub fn push_param(&mut self, e:Expr){
        if let Some(ref mut p) = self.params{
            p.push(e); 
            return 
        }
        self.params = Some(vec![e]);
    }
    
    pub fn eval(&self, scope: &Scope) -> Result<Option<Value>, ActionError> {
        let mut proto = Proto::new().deref(self.d);
        if self.dot { proto = proto.dot()}
        if self.var { proto = proto.var()}
        for e in self.exs {
            proto.push_val(e.eval(scope)?);
        }

        //resolve proto to value
        let mut derefs = proto.derefs;

        let mut val = None;
        while derefs > 0{
            match scope.get(&proto){
                Some(Value::Proto(np)) => {
                    derefs = derefs + np.derefs -1;
                    proto = np.with_set_deref(derefs);
                },
                Some(v) => {
                    return Ok(Some(v.clone()));
                }
                None =>{
                    return Err(ActionError::new("proto points to nothing"));
                }
            }
        }
        match val {
            Some(Value::FuncDef(pnames,actions))=>if let Some(pv) = self.params{
                let mut params = Vec::new();
                for p in pv{
                    params.push(p.eval(scope)?);
                }
                match proto.as_func_name() {
                    "d" => return api_funcs::d(self, &params),
                    "foreach" => return api_funcs::for_each(self, params),
                    "fold" => return api_funcs::fold(self, params),
                    "load" => return api_funcs::load(self, params),
                    "link" => return api_funcs::link(self, params),
                    "if" => return api_funcs::if_expr(self, params),
                    _ => {}
                }
                scope
                    .run_func(&pnames,&actions, &params)?
                    .ok_or(ActionError::new("func in expr returns no value"))
            }else {

            }
            Some(Value::ExprDef(e))=> if let Some(pv) = self.params{// has brackets
                return e.eval(scope);
            }else {return Ok(Value::ExprDef(e))}
            Some(v) => v,

        }
        if let Some(pv) = self.params(){
            params = Vec::new();
            for p in pv{
                params.push(p.eval(scope)?);
            }
        }
           
    }


    pub fn from_tokens(t: &mut TokPrev) -> Result<Self, LineError> {
        let mut res = ProtoX{
            d:0,
            dot:false,
            var:false,
            exs:Vec::new(),
            params:None,
        };


        match t.next() {
            Some(Token::Var)=>res.var = true,
            Some(Token::Dot)=>res.dot = true,
            _=>t.back()
        }
        while let Some(Token::Dollar) = t.next(){
            res.d +=1;
        }
        t.back();

        while let Some(v) = t.next() {
            match v {
                Token::Dot => return Err(t.err("Double Dot")),
                Token::Qoth(s) | Token::Ident(s) => res.exs.push(Expr::Val(Value::Str(s))),
                Token::Num(n) => res.exs.push(Expr::num(n)),
                _ => {
                    t.back();
                    res.exs.push(Expr::from_tokens(t)?);
                }
            }

            match t.next(){
                Some(Token::Dot)=>{},
                Some(Token::BracketO) => {
                    res.params = Some(Vec::new());
                    let mut params = Vec::new();
                    while let Some(tk) = t.next() {
                        match tk {
                            Token::BracketC => return Ok(res),
                            Token::Comma => {}
                            _ => {
                                t.back();
                                res.push_param(Expr::from_tokens(t)?);
                            }
                        }
                    }
                    return Err(t.eof());
                }
                _=>{
                    t.back();
                    return Ok(res);
                }

            }
        }
        Ok(res)
    }
}
