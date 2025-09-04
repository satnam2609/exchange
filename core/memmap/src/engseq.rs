use serde::{Deserialize, Serialize};

///
/// This struct is will be used to send data between sequencer and the matching engine.
/// As the sequencer will try to sequence incoming orders and assign some sequence ids for
/// event sourcing by the matching engine.
//
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawSequencedOrder {
    pub seq_id: u128,
    pub order_id: String,
    pub quote: String,
    pub price: f64,
    pub size: u64,
    pub side: bool,
    pub order_type: bool,
}

impl Default for RawSequencedOrder {
    fn default() -> Self {
        RawSequencedOrder {
            seq_id: 0,
            order_id: "DEFAULT_ORDER".into(),
            quote: "DEFAULT".into(),
            price: 0.0,
            size: 0,
            side: false,
            order_type: false,
        }
    }
}


impl RawSequencedOrder{
    pub fn with_seq_id(&mut self,id:u128)->&mut Self{
        self.seq_id=id;
        self
    }

    pub fn with_order_id(&mut self,order_id:String)->&mut Self{
        self.order_id=order_id;
        self
    }

    pub fn with_quote(&mut self,quote:String)->&mut Self{
        self.quote=quote;
        self
    }


    pub fn with_price(&mut self,price:f64)->&mut Self{
        self.price=price;
        self
    }

    pub fn with_size(&mut self,size:u64)->&mut Self{
        self.size=size;
        self
    }

    pub fn with_side(&mut self,side:bool)->&mut Self{
        self.side=side;
        self
    }

    pub fn with_order_type(&mut self,order_type:bool)->&mut Self{
        self.order_type=order_type;
        self
    }
}
