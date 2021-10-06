//! All the standard operators.

mod optrait;
pub use optrait::*;

mod opext;
pub use opext::*;



mod nullop;
pub use nullop::*;

mod constop;
pub use constop::*;

mod onceop;
pub use onceop::*;

mod iterop;
pub use iterop::*;

mod debugop;
pub use debugop::*;

mod debottomop;
pub use debottomop::*;

mod dynop;
pub use dynop::*;

mod latticeop;
pub use latticeop::*;

mod morphop;
pub use morphop::*;

mod splitop;
pub use splitop::*;

mod switchop;
pub use switchop::*;

mod mergeop;
pub use mergeop::*;

mod binaryop;
pub use binaryop::*;

mod readop;
pub use readop::*;

mod zipop;
pub use zipop::*;

mod channelop;
pub use channelop::*;

mod tcpop;
pub use tcpop::*;

mod tcpserverop;
pub use tcpserverop::*;

mod batchconvertop;
pub use batchconvertop::*;

mod topop;
pub use topop::*;
