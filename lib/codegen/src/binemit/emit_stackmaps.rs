use ir::Function;
use ir::dfg;
use isa::TargetIsa;
use regalloc::RegDiversions;
use ir::instructions::{BranchInfo, CallInfo, InstructionData};
use ir::types;
use ir::{Ebb, FuncRef, Inst, SigRef, Signature, Type, Value, ValueList, ValueListPool};

// This function needs to know what function call we are talking about so that it can
// properly print out the arguments, the live variables, etc.
pub fn emit_stackmaps(inst: Inst) {
    // display_inst(inst);
    println!("Hola mi amigos!");
}



// pub fn display_inst<'a, I: Into<Option<&'a TargetIsa>>>(&'a self, inst: Inst, isa: I,) -> DisplayInst<'a>
// {
//     DisplayInst(self, isa.into(), inst)
// }


// /// Pick the smallest valid encodings for instructions.
// pub fn shrink_instructions(func: &mut Function, isa: &TargetIsa) {
//     let encinfo = isa.encoding_info();
//     let mut divert = RegDiversions::new();
//
//     for ebb in func.layout.ebbs() {
//         divert.clear();
//         for inst in func.layout.ebb_insts(ebb) {
//             let enc = func.encodings[inst];
//             if enc.is_legal() {
//                 let ctrl_type = func.dfg.ctrl_typevar(inst);
//
//                 // Pick the last encoding with constraints that are satisfied.
//                 let best_enc = isa.legal_encodings(func, &func.dfg[inst], ctrl_type)
//                     .filter(|e| {
//                         encinfo.constraints[e.recipe()].satisfied(inst, &divert, &func)
//                     })
//                     .min_by_key(|e| encinfo.bytes(*e))
//                     .unwrap();
//
//                 if best_enc != enc {
//                     func.encodings[inst] = best_enc;
//
//                     dbg!(
//                         "Shrunk [{}] to [{}] in {}, reducing the size from {} to {}",
//                         encinfo.display(enc),
//                         encinfo.display(best_enc),
//                         func.dfg.display_inst(inst, isa),
//                         encinfo.bytes(enc),
//                         encinfo.bytes(best_enc)
//                     );
//                 }
//
//             }
//             divert.apply(&func.dfg[inst]);
//         }
//     }
// }
