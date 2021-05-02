pub mod rendering;
pub mod v2;

#[macro_use]
extern crate log;

pub fn get_text() -> String {
    String::from(
        ":theme Default
 :title Example Sequence Diagram
 :author Mr. Sequence Diagram
 :date

 # rox example
Sigma -> ROX: Order
ROX -> IR: Waves
IR -> TW: RFQ
TW -> ROX: RFQ
ROX -> Sigma: Fills

 # diagram
 #Client -> Server: Request
 #Server -> Server: Parses request
 #Server ->> Service: Query
 #Service -->> Server: Data
 #Server --> Client: Response
 #Left -> Right
 #{AMPS} -> Client: ",
    )
}
