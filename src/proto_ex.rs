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
    pub fn new()->Self{
        ProtoX{
            d:0,
            dot:false,
            var:false,
            exs:Vec::new(),
            params:None,
        }
    }

    pub fn push(mut self,s:&str)->Self{
        self.exs.push(Expr::Val(Value::Str(s.to_string())));
        self
            
    }
    pub fn dot(mut self)->Self{
        self.dot = true;
        self
    }

    pub fn deref(mut self,n:i32)->Self{
        self.d += n;
        self
    }
    pub fn push_param(&mut self, e:Expr){
        if let Some(ref mut p) = self.params{
            p.push(e); 
            return 
        }
        self.params = Some(vec![e]);
    }

    pub fn eval_expr(&self, scope:&Scope)->Result<Value,ActionError>{
        match scope.on_wrap(|wrap| self.eval_mut(wrap)) {
            Ok(None)=>Err(ActionError::new("Expr did not return a value")),
            Ok(Some(v))=>Ok(v),
            Err(e)=>Err(e),
        }
    }
    
    pub fn eval_mut(&self, scope: &mut Scope) -> Result<Option<Value>, ActionError> {
        let mut proto = Proto::new().deref(self.d);
        if self.dot { proto = proto.dot()}
        if self.var { proto = proto.var()}
        for e in &self.exs {
            proto.push_val(e.eval(scope)?)?;
        }

        let param_vals = match &self.params {
            Some(pp) => {
                let mut res = Vec::new();
                for p in pp {
                    res.push(p.eval(scope)?);
                }
                Some(res)
            }
            _=>None,
        };

        if let Some(apf) = proto.as_api_func_name(){
            if let Some(ref pv) = param_vals{
                if let Some(res) = api_funcs::run_api_expr(apf,scope,pv){
                    return res;
                }
            }
        }
        //resolve proto to value
        let mut derefs = proto.derefs;

        let mut val = None  ;
        while derefs > 0{
            match scope.get(&proto){
                Some(Value::Proto(np)) => {
                    derefs = derefs + np.derefs -1;
                    proto = np.with_set_deref(derefs);
                },
                Some(v) => {
                    val = Some(v.clone());
                    break;
                }
                None =>{
                    return Err(ActionError::new("proto points to nothing"));
                }
            }
        }
        if val.is_none() {
            return Ok(Some(Value::Proto(proto)));
        }
        match val {
            Some(Value::FuncDef(pnames,actions))=>if let Some(ref pv) = param_vals{
                scope
                    .run_func(&pnames,&actions, pv)
            }else {
                Ok(Some(Value::FuncDef(pnames.clone(),actions.clone())))

            }
            Some(Value::ExprDef(e))=> if let Some(ref _pv) = self.params{// has brackets
                e.eval(scope).map(|v| Some(v))
            }else {return Ok(Some(Value::ExprDef(e.clone())))}
            Some(r)=>Ok(Some(r.clone())),
            None => Ok(None),

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
