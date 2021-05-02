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

 # diagram
 Client -> Server: Request
 Server -> Service: Handle request 
 Service ->> Database: Query
 Database -->> Service: Data
 Service --> Server: Response
 Server -> Client",
    )
}
