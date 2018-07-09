// use ir::dfg;
use ir::Function;
use ir::types;
use isa::TargetIsa;
use regalloc::RegDiversions;
use dominator_tree::DominatorTree;
use regalloc::live_value_tracker::LiveValueTracker;
use regalloc::liveness::Liveness;

/// This function needs to know what function call we are talking about so that it can
/// properly print out the arguments, the live variables, etc.
pub fn emit_stackmaps(isa: &TargetIsa, func: & Function, domtree: &mut DominatorTree,
    liveness: &mut Liveness, tracker: &mut LiveValueTracker) {
    println!("--------------------------------------------------------------");
    println!("Stackmap Information");
    println!("--------------------------------------------------------------");

    let mut i = 0;

    // Follow shrink_instructions in shrink.rs
    let mut divert = RegDiversions::new();

    // Post-order traversal here?
    for ebb in func.layout.ebbs() {
        divert.clear();

        for inst in func.layout.ebb_insts(ebb) {
            let enc = func.encodings[inst];

            if enc.is_legal() {
                // function signature: emit_stackmaps(isa, func, domtree, &self.liveness, &self.tracker);

                // grab type
                let ctrl_type = func.dfg.ctrl_typevar(inst);

                // check if type is R32
                if ctrl_type == types::R32 {
                    println!("In Instruction {}: {}", i, func.dfg.display_inst(inst, isa));
                    println!("  Controlling Type: {:?}", ctrl_type);
                    println!("      in Registers: {:?}", func.dfg.inst_args(inst));
                    println!("      with Results: {:?}", func.dfg.inst_results(inst));
                    // add result args
                }

                i = i + 1;
            }
            divert.apply(&func.dfg[inst]);
        }
    }
    println!("--------------------------------------------------------------");
}
