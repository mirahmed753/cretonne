// use ir::dfg;
use ir::{Function, types, Type, Value};
use isa::TargetIsa;
use regalloc::RegDiversions;
use dominator_tree::DominatorTree;
use regalloc::live_value_tracker::LiveValueTracker;
use regalloc::liveness::Liveness;
use scoped_hash_map::ScopedHashMap;
use ir::entities::Inst;
use cursor::{Cursor, FuncCursor};

// emit_stackmaps() function prints the Instruction and the liveness of its defs and ops
// by traversing the dominator tree in a post order fashion
pub fn emit_stackmaps(isa: &TargetIsa, func: &mut Function, domtree: &mut DominatorTree,
    liveness: &mut Liveness, tracker: &mut LiveValueTracker) {

    // ScopedHashMap will contain a Cretonne Value as a <Key> and a boolean as
    // its <Value> in order to check liveness of the Cretonne values
    let mut visible_values: ScopedHashMap<Value, bool> = ScopedHashMap::new();
    let mut scope_stack: Vec<Inst> = Vec::new();

    // Visit EBBs in post-order
    // A `FuncCursor` holds a mutable reference to a whole `ir::Function` while keeping a position
    // too. The function can be re-borrowed by accessing the public `cur.func` member.
    let mut pos = FuncCursor::new(func);
    let mut i = 1;

    for &ebb in domtree.cfg_postorder().iter() {
        // Pop any scopes that we just exited.
        loop {
            if let Some(current) = scope_stack.last() {
                if domtree.dominates(*current, ebb, &pos.func.layout) {
                    break;
                }
            } else {
                break;
            }
            scope_stack.pop();
            visible_values.decrement_depth();
        }

        // Push a scope for the current block.
        scope_stack.push(pos.func.layout.first_inst(ebb).unwrap());
        visible_values.increment_depth();

        // From the top of the ebb, step through the instructions
        pos.goto_top(ebb);

        let mut def_list: Vec<Value> = Vec::new();
        let mut op_list: Vec<Value> = Vec::new();

        while let Some(inst) = pos.next_inst() {
            // For each instruction, we expect the operands to be in the SHM,
            // but we do not expect the defs to be in there
            println!("Instruction {}: {}", i, pos.func.dfg.display_inst(inst, isa));

            def_list = pos.func.dfg.inst_results(inst).to_vec();
            op_list = pos.func.dfg.inst_args(inst).to_vec();

            i = i + 1;

            // get liveness of the items in def
            println!("     Def Liveness:");
            for def_item in def_list {
                // get their liveness
                let lr = &liveness[def_item];
                println!("          {}: {:?}", def_item, lr.def_local_end());
            }

            // get liveness of the items in op
            println!("     Op Liveness:");
            for op_item in op_list {
                // get their liveness
                let lr = &liveness[op_item];
                println!("          {}: {:?}", op_item, lr.def_local_end());
            }
        }
    }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

// /// This function needs to know what function call we are talking about so that it can
// /// properly print out the arguments, the live variables, etc.
// pub fn emit_stackmaps_normal_traversal(isa: &TargetIsa, func: &mut Function, domtree: &mut DominatorTree,
//     liveness: &mut Liveness, tracker: &mut LiveValueTracker) {
//
//     // ScopedHashMap will contain a Cretonne Value as a <Key> and a boolean as
//     // its <Value> in order to check liveness of the Cretonne values
//     // let mut visible_values: ScopedHashMap<Value, bool> = ScopedHashMap::new();
//     // let mut scope_stack: Vec<Inst> = Vec::new();
//
//     println!("--------------------------------------------------------------");
//     println!("Stackmap Information");
//     println!("--------------------------------------------------------------");
//
//     let mut i = 0;
//
//     // Follow shrink_instructions in shrink.rs
//     let mut divert = RegDiversions::new();
//
//     // Post-order traversal here?
//     for ebb in func.layout.ebbs() {
//         divert.clear();
//
//         // print the entire ebb
//         println!("{:?}", ebb);
//
//         for inst in func.layout.ebb_insts(ebb) {
//
//             let enc = func.encodings[inst];
//
//             // create vectors to hold the defs and ops from the instructions
//             let mut def_list: Vec<Value> = Vec::new();
//             let mut op_list: Vec<Value> = Vec::new();
//
//             if enc.is_legal() {
//                 // function signature: emit_stackmaps(isa, func, domtree, &self.liveness, &self.tracker);
//
//                 // grab type
//                 let ctrl_type = func.dfg.ctrl_typevar(inst);
//                 println!("In Instruction {}: {}", i, func.dfg.display_inst(inst, isa));
//                 // check if type is R32
//                 if ctrl_type == types::I32 {
//                     // println!("In Instruction {}: {}", i, func.dfg.display_inst(inst, isa));
//
//                     // For testing purposes
//                     println!("     The DEFS are: {:?}", func.dfg.inst_results(inst));
//                     println!("     The OPS are: {:?}", func.dfg.inst_args(inst));
//
//                     def_list = func.dfg.inst_results(inst).to_vec();
//                     op_list = func.dfg.inst_args(inst).to_vec();
//
//                 }
//
//                 i = i + 1;
//             }
//             divert.apply(&func.dfg[inst]);
//
//             // let ctx = self.liveness.context(&self.cur.func.layout);
//
//             if func.dfg.ctrl_typevar(inst) == types::I32 {
//                 // get liveness of the items in def
//                 println!("     Def Liveness:");
//                 for def_item in def_list {
//                     // get their liveness
//                     let lr = &liveness[def_item];
//                     println!("          {}: {:?}", def_item, lr.def_local_end());
//                 }
//
//                 // get liveness of the items in op
//                 println!("     Op Liveness:");
//                 for op_item in op_list {
//                     // get their liveness
//                     let lr = &liveness[op_item];
//                     println!("          {}: {:?}", op_item, lr.def_local_end());
//                 }
//                 println!();
//             }
//         } // end inst for loop
//     } // end ebb for loop
//     println!("--------------------------------------------------------------");
// }

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

// pub fn emit_stackmaps_old(isa: &TargetIsa, func: &mut Function, domtree: &mut DominatorTree,
//     liveness: &mut Liveness, tracker: &mut LiveValueTracker) {
//
//     // ScopedHashMap will contain a Cretonne Value as a <Key> and a boolean as
//     // its <Value> in order to check liveness of the Cretonne values
//     let mut visible_values: ScopedHashMap<Value, bool> = ScopedHashMap::new();
//     let mut scope_stack: Vec<Inst> = Vec::new();
//
//     // Visit EBBs in post-order
//     // A `FuncCursor` holds a mutable reference to a whole `ir::Function` while keeping a position
//     // too. The function can be re-borrowed by accessing the public `cur.func` member.
//     let mut pos = FuncCursor::new(func);
//
//     for &ebb in domtree.cfg_postorder().iter() {
//         // Pop any scopes that we just exited.
//         loop {
//             if let Some(current) = scope_stack.last() {
//                 if domtree.dominates(*current, ebb, &pos.func.layout) {
//                     break;
//                 }
//             } else {
//                 break;
//             }
//             scope_stack.pop();
//             visible_values.decrement_depth();
//         }
//
//         // Push a scope for the current block.
//         scope_stack.push(pos.func.layout.first_inst(ebb).unwrap());
//         visible_values.increment_depth();
//
//         pos.goto_top(ebb);
//         while let Some(inst) = pos.next_inst() {
//             // Resolve aliases, particularly aliases we created earlier.
//             pos.func.dfg.resolve_aliases_in_arguments(inst);    // Find original SSA values being aliased
//
//             // grab controlling type (Key)
//             let ctrl_typevar = pos.func.dfg.ctrl_typevar(inst);
//             let key = ctrl_typevar;
//             let entry = visible_values.entry(key);
//             use scoped_hash_map::Entry::*;
//             match entry {
//                 Occupied(entry) => {
//                     // debug_assert!(domtree.dominates(*entry.get(), inst, &pos.func.layout));
//                     // // If the redundant instruction is representing the current
//                     // // scope, pick a new representative.
//                     // let old = scope_stack.last_mut().unwrap();
//                     // if *old == inst {
//                     //     *old = pos.func.layout.next_inst(inst).unwrap();
//                     // }
//                     // // Replace the redundant instruction and remove it.
//                     // pos.func.dfg.replace_with_aliases(inst, *entry.get());
//                     // pos.remove_inst_and_step_back();
//                     println!("{:?}", entry);
//                 }
//                 // insert entry with boolean value: true
//                 Vacant(entry) => {
//                     if key == types::R32 {
//                         entry.insert(true);
//                     }
//                 }
//             }
//         }
//     }
//
//     // print contents of ScopedHashMap
// }
