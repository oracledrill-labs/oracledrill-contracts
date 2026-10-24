#![no_std]
use soroban_sdk::{contract,contractimpl,contracttype,contracterror,Address,Env,Symbol};
#[contracttype]
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Report {pub feed:Symbol,pub answer:i128,pub decimals:u32,pub timestamp:u64}
#[contracttype]
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Policy {pub feed:Symbol,pub decimals:u32,pub max_age:u64,pub future_skew:u64,pub max_deviation_bps:u32,pub require_reference:bool}
#[contracttype]
#[derive(Clone)]
enum Key {Admin,Policy,Report}
#[contracterror]
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
#[repr(u32)]
pub enum Error {AlreadyInitialized=1,NotInitialized=2,InvalidPolicy=3,MissingReport=4,WrongFeed=5,NonPositive=6,WrongDecimals=7,InvalidTimestamp=8,Future=9,Stale=10,MissingReference=11,Divergence=12,Arithmetic=13}
fn validate(r:&Report,p:&Policy,now:u64)->Result<(),Error>{
 if r.feed!=p.feed{return Err(Error::WrongFeed)}
 if r.answer<=0{return Err(Error::NonPositive)}
 if r.decimals!=p.decimals{return Err(Error::WrongDecimals)}
 if r.timestamp==0{return Err(Error::InvalidTimestamp)}
 if r.timestamp>now && r.timestamp-now>p.future_skew{return Err(Error::Future)}
 if now>r.timestamp && now-r.timestamp>p.max_age{return Err(Error::Stale)}
 Ok(())
}
pub fn evaluate(r:&Report,p:&Policy,reference:Option<Report>,now:u64)->Result<i128,Error>{
 if p.decimals>18||p.max_age==0||p.max_age>86400||p.future_skew>300||p.max_deviation_bps>10000{return Err(Error::InvalidPolicy)}
 validate(r,p,now)?;
 if p.require_reference&&reference.is_none(){return Err(Error::MissingReference)}
 if let Some(other)=reference {validate(&other,p,now)?;let diff=r.answer.checked_sub(other.answer).ok_or(Error::Arithmetic)?.checked_abs().ok_or(Error::Arithmetic)?;
 let lhs=diff.checked_mul(10000).ok_or(Error::Arithmetic)?;let rhs=other.answer.checked_mul(p.max_deviation_bps as i128).ok_or(Error::Arithmetic)?;
 if lhs>rhs{return Err(Error::Divergence)}}
 Ok(r.answer)
}
#[contract]
pub struct OracleDrill;
#[contractimpl]
impl OracleDrill {
 pub fn __constructor(env:Env,admin:Address,policy:Policy)->Result<(),Error>{
  if env.storage().instance().has(&Key::Admin){return Err(Error::AlreadyInitialized)}
  admin.require_auth();
  if policy.decimals>18||policy.max_age==0||policy.max_age>86400||policy.future_skew>300||policy.max_deviation_bps>10000{return Err(Error::InvalidPolicy)}
  env.storage().instance().set(&Key::Admin,&admin);env.storage().instance().set(&Key::Policy,&policy);Ok(())
 }
 // This is an administrator-controlled MOCK report, not an authenticated oracle adapter.
 pub fn set_report(env:Env,report:Report)->Result<(),Error>{let admin:Address=env.storage().instance().get(&Key::Admin).ok_or(Error::NotInitialized)?;admin.require_auth();env.storage().instance().set(&Key::Report,&report);Ok(())}
 pub fn quote_unsafe(env:Env)->Result<i128,Error>{let r:Report=env.storage().instance().get(&Key::Report).ok_or(Error::MissingReport)?;Ok(r.answer)}
 pub fn quote_guarded(env:Env,reference:Option<Report>)->Result<i128,Error>{let p:Policy=env.storage().instance().get(&Key::Policy).ok_or(Error::NotInitialized)?;let r:Report=env.storage().instance().get(&Key::Report).ok_or(Error::MissingReport)?;evaluate(&r,&p,reference,env.ledger().timestamp())}
 pub fn current(env:Env)->Option<Report>{env.storage().instance().get(&Key::Report)}
}
#[cfg(test)]
mod test {
 use super::*;use soroban_sdk::{symbol_short,testutils::{Address as _,Ledger}};
 fn policy()->Policy{Policy{feed:symbol_short!("XLMUSD"),decimals:8,max_age:60,future_skew:5,max_deviation_bps:500,require_reference:false}}
 fn report()->Report{Report{feed:symbol_short!("XLMUSD"),answer:10000000,decimals:8,timestamp:1000}}
 #[test]fn fresh(){assert_eq!(evaluate(&report(),&policy(),None,1000),Ok(10000000))}
 #[test]fn stale(){assert_eq!(evaluate(&report(),&policy(),None,1061),Err(Error::Stale))}
 #[test]fn max_age_boundary(){assert!(evaluate(&report(),&policy(),None,1060).is_ok())}
 #[test]fn future(){assert_eq!(evaluate(&report(),&policy(),None,994),Err(Error::Future))}
 #[test]fn wrong_decimals(){let mut r=report();r.decimals=7;assert_eq!(evaluate(&r,&policy(),None,1000),Err(Error::WrongDecimals))}
 #[test]fn negative(){let mut r=report();r.answer=-1;assert_eq!(evaluate(&r,&policy(),None,1000),Err(Error::NonPositive))}
 #[test]fn missing_reference(){let mut p=policy();p.require_reference=true;assert_eq!(evaluate(&report(),&p,None,1000),Err(Error::MissingReference))}
 #[test]fn divergent(){let mut r=report();r.answer=12000000;assert_eq!(evaluate(&r,&policy(),Some(report()),1000),Err(Error::Divergence))}
 #[test]fn overflow(){let mut r=report();r.answer=i128::MAX;assert_eq!(evaluate(&r,&policy(),Some(report()),1000),Err(Error::Arithmetic))}
 #[test]fn reference_must_be_fresh(){let mut r=report();r.timestamp=1061;assert_eq!(evaluate(&r,&policy(),Some(report()),1061),Err(Error::Stale))}
 #[test]fn integration_unsafe_accepts_stale_guarded_rejects(){let env=Env::default();env.mock_all_auths();env.ledger().with_mut(|l|l.timestamp=1061);let admin=Address::generate(&env);let id=env.register(OracleDrill,(&admin,&policy()));let client=OracleDrillClient::new(&env,&id);client.set_report(&report());assert_eq!(client.quote_unsafe(),10000000);assert_eq!(client.try_quote_guarded(&None),Err(Ok(Error::Stale)))}
 #[test]#[should_panic]fn unauthorized_set_report_rejected(){let env=Env::default();let admin=Address::generate(&env);env.mock_all_auths();let id=env.register(OracleDrill,(&admin,&policy()));let client=OracleDrillClient::new(&env,&id);env.mock_auths(&[]);client.set_report(&report());}
}
