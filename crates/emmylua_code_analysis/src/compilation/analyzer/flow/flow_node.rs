use std::collections::HashMap;

use emmylua_parser::{
    LuaAst, LuaAstNode, LuaBlock, LuaGotoStat, LuaLabelStat, LuaStat, LuaSyntaxId,
};
use rowan::{TextRange, TextSize};
use smol_str::SmolStr;

use crate::LuaFlowId;

#[derive(Debug)]
pub struct FlowNode {
    flow_id: LuaFlowId,
    parent_id: Option<LuaFlowId>,
    label_ref: HashMap<BlockId, Vec<(SmolStr, LuaLabelStat)>>,
    jump_to_stat_end: HashMap<LuaSyntaxId, LuaStat>,
    children: Vec<LuaFlowId>,
    range: TextRange,
}

#[allow(unused)]
impl FlowNode {
    pub fn new(flow_id: LuaFlowId, range: TextRange, parent_id: Option<LuaFlowId>) -> FlowNode {
        FlowNode {
            flow_id,
            parent_id,
            children: Vec::new(),
            label_ref: HashMap::new(),
            jump_to_stat_end: HashMap::new(),
            range,
        }
    }

    pub fn get_range(&self) -> TextRange {
        self.range
    }

    pub fn get_flow_id(&self) -> LuaFlowId {
        self.flow_id
    }

    pub fn get_parent_id(&self) -> Option<LuaFlowId> {
        self.parent_id
    }

    pub fn get_children(&self) -> &Vec<LuaFlowId> {
        &self.children
    }

    pub fn add_child(&mut self, child: LuaFlowId) {
        self.children.push(child);
    }

    pub fn add_label_ref(&mut self, name: &str, label: LuaLabelStat) -> Option<()> {
        let block = label.get_parent::<LuaBlock>()?;
        let block_id = BlockId::from_block(block);
        let name = SmolStr::new(name);

        self.label_ref
            .entry(block_id)
            .or_insert_with(Vec::new)
            .push((name, label));

        Some(())
    }

    pub fn is_exist_label_in_same_block(&self, name: &str, block_id: BlockId) -> bool {
        let name = SmolStr::new(name);
        self.label_ref
            .get(&block_id)
            .map_or(false, |labels| labels.iter().any(|(n, _)| n == &name))
    }

    pub fn find_label(&self, name: &str, goto: LuaGotoStat) -> Option<&LuaLabelStat> {
        let name = SmolStr::new(name);
        for block in goto.ancestors::<LuaBlock>() {
            let block_id = BlockId::from_block(block);
            if block_id.0 < self.flow_id.get_position() {
                break;
            }

            if let Some(labels) = self.label_ref.get(&block_id) {
                for (label_name, label) in labels {
                    if label_name == &name {
                        return Some(label);
                    }
                }
            }
        }

        None
    }

    pub fn add_jump_to_stat(&mut self, jump_syntax_id: LuaSyntaxId, stat: LuaStat) {
        self.jump_to_stat_end.insert(jump_syntax_id, stat);
    }

    pub fn get_jump_to_stat(&self, jump_syntax_id: LuaSyntaxId) -> Option<LuaStat> {
        self.jump_to_stat_end.get(&jump_syntax_id).cloned()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct BlockId(TextSize);

impl BlockId {
    pub fn from_block(block: LuaBlock) -> BlockId {
        BlockId(block.get_position())
    }

    pub fn from_ast(ast: LuaAst) -> Option<BlockId> {
        if LuaBlock::can_cast(ast.syntax().kind().into()) {
            Some(BlockId(ast.get_position()))
        } else {
            let block = ast.ancestors::<LuaBlock>().next()?;
            Some(BlockId(block.get_position()))
        }
    }
}
