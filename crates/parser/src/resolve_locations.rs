//! Implements the `resolve_locations` function, which derives the location of a node if not set on
//! the node itself.

use crate::get_children_codegen::ChildrenNode;
use crate::get_location_codegen::get_location;
use pg_query::NodeEnum;
use regex::Regex;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct NestedNode {
    pub node: NodeEnum,
    pub depth: i32,
    pub location: i32,
    pub path: String,
}

/// Resolves the location of `ChildrenNode`
///
/// Uses the `.location` property if available on the node, and otherwise tries to
/// derive the location manually.
pub fn resolve_locations(children: Vec<ChildrenNode>, text: &str) -> Vec<NestedNode> {
    let mut nodes: Vec<NestedNode> = vec![];

    let mut stack: VecDeque<ChildrenNode> = VecDeque::from(children);

    while !stack.is_empty() {
        let current_node = stack.pop_front().unwrap();

        // if location is set, we can skip the rest
        let location = get_location(&current_node.node);
        if location.is_some() {
            nodes.push(NestedNode {
                node: current_node.node,
                depth: current_node.depth,
                location: location.unwrap(),
                path: current_node.path.clone(),
            });
            continue;
        }

        // get parent node and its location
        let parent_node = nodes.iter().find(|n| {
            let mut path_elements = current_node.path.split(".").collect::<Vec<&str>>();
            path_elements.pop();
            let parent_path = path_elements.join(".");
            n.path == parent_path
        });

        let parent_location = if parent_node.is_some() {
            parent_node.unwrap().location
        } else {
            // fallback to 0 if no parent node is found
            0
        };

        // we cannot assume that for each node, there is at least one child with a location.
        // e.g. in `DROP TABLE tablename;`, the location of the `List` node that wraps the String node `tablename` is not known
        let earliest_child_location = nodes
            .iter()
            .filter(|n| n.path.starts_with(current_node.path.as_str()))
            .min_by(|a, b| a.location.cmp(&b.location))
            .map(|n| n.location);

        let location = derive_location(
            &current_node.node,
            text.clone(),
            parent_location,
            earliest_child_location,
        );

        if location.is_some() {
            nodes.push(NestedNode {
                node: current_node.node,
                depth: current_node.depth,
                location: location.unwrap(),
                path: current_node.path.clone(),
            });
        } else if stack
            .iter()
            .find(|x| x.path.starts_with(current_node.path.as_str()))
            .is_some()
        {
            // if there are still children to be processed, we push the node back to the stack and
            // try again later in the hope that we could find the location for a children node of
            // the current node
            stack.push_back(current_node);
        }
    }

    nodes
}

fn derive_location(
    // The node to derive the location for
    node: &NodeEnum,
    // The full text of the query
    text: &str,
    // The location of the parent node
    parent_location: i32,
    // not given if node does not have any children
    earliest_child_location: Option<i32>,
) -> Option<i32> {
    match node {
        NodeEnum::Alias(_) => todo!(),
        NodeEnum::RangeVar(_) => panic!("Node has location property."),
        NodeEnum::TableFunc(_) => panic!("Node has location property."),
        NodeEnum::Var(_) => panic!("Node has location property."),
        NodeEnum::Param(_) => panic!("Node has location property."),
        NodeEnum::Aggref(_) => panic!("Node has location property."),
        NodeEnum::GroupingFunc(_) => panic!("Node has location property."),
        NodeEnum::WindowFunc(_) => panic!("Node has location property."),
        NodeEnum::SubscriptingRef(_) => todo!(),
        NodeEnum::FuncExpr(_) => panic!("Node has location property."),
        NodeEnum::NamedArgExpr(_) => panic!("Node has location property."),
        NodeEnum::OpExpr(_) => panic!("Node has location property."),
        NodeEnum::DistinctExpr(_) => panic!("Node has location property."),
        NodeEnum::NullIfExpr(_) => panic!("Node has location property."),
        NodeEnum::ScalarArrayOpExpr(_) => panic!("Node has location property."),
        NodeEnum::BoolExpr(_) => panic!("Node has location property."),
        NodeEnum::SubLink(_) => panic!("Node has location property."),
        NodeEnum::SubPlan(_) => todo!(),
        NodeEnum::AlternativeSubPlan(_) => todo!(),
        NodeEnum::FieldSelect(_) => todo!(),
        NodeEnum::FieldStore(_) => todo!(),
        NodeEnum::RelabelType(_) => panic!("Node has location property."),
        NodeEnum::CoerceViaIo(_) => panic!("Node has location property."),
        NodeEnum::ArrayCoerceExpr(_) => panic!("Node has location property."),
        NodeEnum::ConvertRowtypeExpr(_) => panic!("Node has location property."),
        NodeEnum::CollateExpr(_) => panic!("Node has location property."),
        NodeEnum::CaseExpr(_) => panic!("Node has location property."),
        NodeEnum::CaseWhen(_) => panic!("Node has location property."),
        NodeEnum::CaseTestExpr(_) => todo!(),
        NodeEnum::ArrayExpr(_) => panic!("Node has location property."),
        NodeEnum::RowExpr(_) => panic!("Node has location property."),
        NodeEnum::RowCompareExpr(_) => todo!(),
        NodeEnum::CoalesceExpr(_) => panic!("Node has location property."),
        NodeEnum::MinMaxExpr(_) => panic!("Node has location property."),
        NodeEnum::SqlvalueFunction(_) => panic!("Node has location property."),
        NodeEnum::XmlExpr(_) => panic!("Node has location property."),
        NodeEnum::NullTest(_) => panic!("Node has location property."),
        NodeEnum::BooleanTest(_) => panic!("Node has location property."),
        NodeEnum::CoerceToDomain(_) => panic!("Node has location property."),
        NodeEnum::CoerceToDomainValue(_) => panic!("Node has location property."),
        NodeEnum::SetToDefault(_) => panic!("Node has location property."),
        NodeEnum::CurrentOfExpr(_) => todo!(),
        NodeEnum::NextValueExpr(_) => todo!(),
        NodeEnum::InferenceElem(_) => todo!(),
        NodeEnum::TargetEntry(_) => todo!(),
        NodeEnum::RangeTblRef(_) => todo!(),
        NodeEnum::JoinExpr(n) => {
            let keyword_regexp = match n.jointype() {
                pg_query::protobuf::JoinType::Undefined => todo!(),
                pg_query::protobuf::JoinType::JoinInner => "join|inner",
                pg_query::protobuf::JoinType::JoinLeft => "join",
                pg_query::protobuf::JoinType::JoinFull => "full",
                pg_query::protobuf::JoinType::JoinRight => "join",
                pg_query::protobuf::JoinType::JoinSemi => todo!(),
                pg_query::protobuf::JoinType::JoinAnti => todo!(),
                pg_query::protobuf::JoinType::JoinUniqueOuter => todo!(),
                pg_query::protobuf::JoinType::JoinUniqueInner => todo!(),
            };

            Some(get_location_via_regexp(
                Regex::new(format!("(?mi){}", keyword_regexp).as_str()).unwrap(),
                text,
                parent_location,
                earliest_child_location,
            ))
        }
        NodeEnum::FromExpr(_) => todo!(),
        NodeEnum::OnConflictExpr(_) => todo!(),
        NodeEnum::IntoClause(_) => todo!(),
        NodeEnum::MergeAction(_) => todo!(),
        NodeEnum::RawStmt(_) => todo!(),
        NodeEnum::Query(_) => todo!(),
        NodeEnum::InsertStmt(_) => Some(get_location_via_regexp(
            Regex::new(r"(?mi)insert\s+into").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::DeleteStmt(_) => Some(get_location_via_regexp(
            Regex::new(r"(?mi)delete\s+from").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::UpdateStmt(_) => todo!(),
        NodeEnum::MergeStmt(_) => todo!(),
        NodeEnum::SelectStmt(_) => Some(get_location_via_regexp(
            // in "insert into contact (id) values (1)" the "values (1)" is a select statement
            Regex::new(r"(?mi)select|values").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::ReturnStmt(_) => todo!(),
        NodeEnum::PlassignStmt(_) => panic!("Node has location property."),
        NodeEnum::AlterTableStmt(_) => Some(get_location_via_regexp(
            Regex::new(r"(?mi)alter\s+table").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::AlterTableCmd(n) => Some(get_location_via_regexp(
            Regex::new(format!("(?mi)alter.*{}", n.name).as_str()).unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::AlterDomainStmt(_) => todo!(),
        NodeEnum::SetOperationStmt(_) => todo!(),
        NodeEnum::GrantStmt(_) => todo!(),
        NodeEnum::GrantRoleStmt(_) => todo!(),
        NodeEnum::AlterDefaultPrivilegesStmt(_) => todo!(),
        NodeEnum::ClosePortalStmt(_) => todo!(),
        NodeEnum::ClusterStmt(_) => todo!(),
        NodeEnum::CopyStmt(_) => todo!(),
        NodeEnum::CreateStmt(_) => todo!(),
        NodeEnum::DefineStmt(_) => todo!(),
        NodeEnum::DropStmt(_) => todo!(),
        NodeEnum::TruncateStmt(_) => todo!(),
        NodeEnum::CommentStmt(_) => todo!(),
        NodeEnum::FetchStmt(_) => todo!(),
        NodeEnum::IndexStmt(_) => todo!(),
        NodeEnum::CreateFunctionStmt(_) => todo!(),
        NodeEnum::AlterFunctionStmt(_) => todo!(),
        NodeEnum::DoStmt(_) => todo!(),
        NodeEnum::RenameStmt(_) => todo!(),
        NodeEnum::RuleStmt(_) => todo!(),
        NodeEnum::NotifyStmt(_) => todo!(),
        NodeEnum::ListenStmt(_) => todo!(),
        NodeEnum::UnlistenStmt(_) => todo!(),
        NodeEnum::TransactionStmt(_) => todo!(),
        NodeEnum::ViewStmt(_) => todo!(),
        NodeEnum::LoadStmt(_) => todo!(),
        NodeEnum::CreateDomainStmt(_) => todo!(),
        NodeEnum::CreatedbStmt(_) => todo!(),
        NodeEnum::DropdbStmt(_) => todo!(),
        NodeEnum::VacuumStmt(_) => todo!(),
        NodeEnum::ExplainStmt(_) => todo!(),
        NodeEnum::CreateTableAsStmt(_) => todo!(),
        NodeEnum::CreateSeqStmt(_) => todo!(),
        NodeEnum::AlterSeqStmt(_) => todo!(),
        NodeEnum::VariableSetStmt(_) => todo!(),
        NodeEnum::VariableShowStmt(_) => todo!(),
        NodeEnum::DiscardStmt(_) => todo!(),
        NodeEnum::CreateTrigStmt(_) => todo!(),
        NodeEnum::CreatePlangStmt(_) => todo!(),
        NodeEnum::CreateRoleStmt(_) => todo!(),
        NodeEnum::AlterRoleStmt(_) => todo!(),
        NodeEnum::DropRoleStmt(_) => todo!(),
        NodeEnum::LockStmt(_) => todo!(),
        NodeEnum::ConstraintsSetStmt(_) => todo!(),
        NodeEnum::ReindexStmt(_) => todo!(),
        NodeEnum::CheckPointStmt(_) => todo!(),
        NodeEnum::CreateSchemaStmt(_) => todo!(),
        NodeEnum::AlterDatabaseStmt(_) => todo!(),
        NodeEnum::AlterDatabaseRefreshCollStmt(_) => todo!(),
        NodeEnum::AlterDatabaseSetStmt(_) => todo!(),
        NodeEnum::AlterRoleSetStmt(_) => todo!(),
        NodeEnum::CreateConversionStmt(_) => todo!(),
        NodeEnum::CreateCastStmt(_) => todo!(),
        NodeEnum::CreateOpClassStmt(_) => todo!(),
        NodeEnum::CreateOpFamilyStmt(_) => todo!(),
        NodeEnum::AlterOpFamilyStmt(_) => todo!(),
        NodeEnum::PrepareStmt(_) => todo!(),
        NodeEnum::ExecuteStmt(_) => todo!(),
        NodeEnum::DeallocateStmt(_) => todo!(),
        NodeEnum::DeclareCursorStmt(_) => todo!(),
        NodeEnum::CreateTableSpaceStmt(_) => todo!(),
        NodeEnum::DropTableSpaceStmt(_) => todo!(),
        NodeEnum::AlterObjectDependsStmt(_) => todo!(),
        NodeEnum::AlterObjectSchemaStmt(_) => todo!(),
        NodeEnum::AlterOwnerStmt(_) => todo!(),
        NodeEnum::AlterOperatorStmt(_) => todo!(),
        NodeEnum::AlterTypeStmt(_) => todo!(),
        NodeEnum::DropOwnedStmt(_) => todo!(),
        NodeEnum::ReassignOwnedStmt(_) => todo!(),
        NodeEnum::CompositeTypeStmt(_) => todo!(),
        NodeEnum::CreateEnumStmt(_) => todo!(),
        NodeEnum::CreateRangeStmt(_) => todo!(),
        NodeEnum::AlterEnumStmt(_) => todo!(),
        NodeEnum::AlterTsdictionaryStmt(_) => todo!(),
        NodeEnum::AlterTsconfigurationStmt(_) => todo!(),
        NodeEnum::CreateFdwStmt(_) => todo!(),
        NodeEnum::AlterFdwStmt(_) => todo!(),
        NodeEnum::CreateForeignServerStmt(_) => todo!(),
        NodeEnum::AlterForeignServerStmt(_) => todo!(),
        NodeEnum::CreateUserMappingStmt(_) => todo!(),
        NodeEnum::AlterUserMappingStmt(_) => todo!(),
        NodeEnum::DropUserMappingStmt(_) => todo!(),
        NodeEnum::AlterTableSpaceOptionsStmt(_) => todo!(),
        NodeEnum::AlterTableMoveAllStmt(_) => todo!(),
        NodeEnum::SecLabelStmt(_) => todo!(),
        NodeEnum::CreateForeignTableStmt(_) => todo!(),
        NodeEnum::ImportForeignSchemaStmt(_) => todo!(),
        NodeEnum::CreateExtensionStmt(_) => todo!(),
        NodeEnum::AlterExtensionStmt(_) => todo!(),
        NodeEnum::AlterExtensionContentsStmt(_) => todo!(),
        NodeEnum::CreateEventTrigStmt(_) => todo!(),
        NodeEnum::AlterEventTrigStmt(_) => todo!(),
        NodeEnum::RefreshMatViewStmt(_) => todo!(),
        NodeEnum::ReplicaIdentityStmt(_) => todo!(),
        NodeEnum::AlterSystemStmt(_) => todo!(),
        NodeEnum::CreatePolicyStmt(_) => todo!(),
        NodeEnum::AlterPolicyStmt(_) => todo!(),
        NodeEnum::CreateTransformStmt(_) => todo!(),
        NodeEnum::CreateAmStmt(_) => todo!(),
        NodeEnum::CreatePublicationStmt(_) => todo!(),
        NodeEnum::AlterPublicationStmt(_) => todo!(),
        NodeEnum::CreateSubscriptionStmt(_) => todo!(),
        NodeEnum::AlterSubscriptionStmt(_) => todo!(),
        NodeEnum::DropSubscriptionStmt(_) => todo!(),
        NodeEnum::CreateStatsStmt(_) => todo!(),
        NodeEnum::AlterCollationStmt(_) => todo!(),
        NodeEnum::CallStmt(_) => todo!(),
        NodeEnum::AlterStatsStmt(_) => todo!(),
        NodeEnum::AExpr(_) => panic!("Node has location property."),
        NodeEnum::ColumnRef(_) => panic!("Node has location property."),
        NodeEnum::ParamRef(_) => panic!("Node has location property."),
        NodeEnum::FuncCall(_) => panic!("Node has location property."),
        NodeEnum::AStar(_) => Some(get_location_via_regexp(
            Regex::new(r"(?mi)\*").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::AIndices(_) => todo!(),
        NodeEnum::AIndirection(_) => todo!(),
        NodeEnum::AArrayExpr(_) => panic!("Node has location property."),
        NodeEnum::ResTarget(_) => panic!("Node has location property."),
        NodeEnum::MultiAssignRef(_) => todo!(),
        NodeEnum::TypeCast(_) => panic!("Node has location property."),
        NodeEnum::CollateClause(_) => panic!("Node has location property."),
        NodeEnum::SortBy(_) => panic!("Node has location property."),
        NodeEnum::WindowDef(_) => panic!("Node has location property."),
        NodeEnum::RangeSubselect(_) => todo!(),
        NodeEnum::RangeFunction(_) => todo!(),
        NodeEnum::RangeTableSample(_) => panic!("Node has location property."),
        NodeEnum::RangeTableFunc(_) => panic!("Node has location property."),
        NodeEnum::RangeTableFuncCol(_) => panic!("Node has location property."),
        NodeEnum::TypeName(_) => panic!("Node has location property."),
        NodeEnum::ColumnDef(_) => panic!("Node has location property."),
        NodeEnum::IndexElem(_) => todo!(),
        NodeEnum::StatsElem(_) => todo!(),
        NodeEnum::Constraint(_) => panic!("Node has location property."),
        NodeEnum::DefElem(_) => panic!("Node has location property."),
        NodeEnum::RangeTblEntry(_) => todo!(),
        NodeEnum::RangeTblFunction(_) => todo!(),
        NodeEnum::TableSampleClause(_) => todo!(),
        NodeEnum::WithCheckOption(_) => todo!(),
        NodeEnum::SortGroupClause(_) => todo!(),
        NodeEnum::GroupingSet(_) => panic!("Node has location property."),
        NodeEnum::WindowClause(_) => todo!(),
        NodeEnum::ObjectWithArgs(_) => todo!(),
        NodeEnum::AccessPriv(n) => Some(get_location_via_regexp(
            Regex::new(format!("(?mi){}", n.priv_name).as_str()).unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )),
        NodeEnum::CreateOpClassItem(_) => todo!(),
        NodeEnum::TableLikeClause(_) => todo!(),
        NodeEnum::FunctionParameter(_) => todo!(),
        NodeEnum::LockingClause(_) => todo!(),
        NodeEnum::RowMarkClause(_) => todo!(),
        NodeEnum::XmlSerialize(_) => panic!("Node has location property."),
        NodeEnum::WithClause(_) => panic!("Node has location property."),
        NodeEnum::InferClause(_) => panic!("Node has location property."),
        NodeEnum::OnConflictClause(_) => panic!("Node has location property."),
        NodeEnum::CtesearchClause(_) => panic!("Node has location property."),
        NodeEnum::CtecycleClause(_) => panic!("Node has location property."),
        NodeEnum::CommonTableExpr(_) => panic!("Node has location property."),
        NodeEnum::MergeWhenClause(_) => todo!(),
        NodeEnum::RoleSpec(n) => {
            if n.location == -1 {
                None
            } else {
                todo!()
            }
        }
        NodeEnum::TriggerTransition(_) => todo!(),
        NodeEnum::PartitionElem(_) => panic!("Node has location property."),
        NodeEnum::PartitionSpec(_) => panic!("Node has location property."),
        NodeEnum::PartitionBoundSpec(_) => panic!("Node has location property."),
        NodeEnum::PartitionRangeDatum(_) => panic!("Node has location property."),
        NodeEnum::PartitionCmd(_) => todo!(),
        NodeEnum::VacuumRelation(_) => todo!(),
        NodeEnum::PublicationObjSpec(_) => panic!("Node has location property."),
        NodeEnum::PublicationTable(_) => todo!(),
        NodeEnum::InlineCodeBlock(_) => todo!(),
        NodeEnum::CallContext(_) => todo!(),
        NodeEnum::Integer(_) => None,
        NodeEnum::Float(_) => None,
        NodeEnum::Boolean(_) => None,
        NodeEnum::String(n) => find_location_via_regexp(
            Regex::new(format!("(?mi){}", n.sval).as_str()).unwrap(),
            text,
            parent_location,
            earliest_child_location,
        ),
        NodeEnum::BitString(_) => None,
        NodeEnum::List(_) => find_location_via_regexp(
            Regex::new(r"(?mi)\((.*?)\)").unwrap(),
            text,
            parent_location,
            earliest_child_location,
        )
        // sometimes, a list is not enclosed by parentheses, but starts at the earliest child
        // location, e.g. `DROP TABLE tablename`, where `tablename` is enclosed by an invisible
        // `List`
        .or(earliest_child_location),
        NodeEnum::IntList(_) => todo!(),
        NodeEnum::OidList(_) => todo!(),
        NodeEnum::AConst(_) => panic!("Node has location property."),
    }
}

fn find_location_via_regexp(
    r: Regex,
    text: &str,
    parent_location: i32,
    earliest_child_location: Option<i32>,
) -> Option<i32> {
    struct Location {
        location: i32,
        distance: i32,
    }

    let location = r
        .find_iter(text)
        .filter_map(|x| {
            if x.start() as i32 >= parent_location {
                Some({
                    Location {
                        location: x.start() as i32,
                        distance: if earliest_child_location.is_some() {
                            earliest_child_location.unwrap() - x.start() as i32
                        } else {
                            x.start() as i32 - parent_location
                        },
                    }
                })
            } else {
                None
            }
        })
        .min_by_key(|x| x.distance.abs());

    if location.is_none() {
        return None;
    }

    let location = location.unwrap().location;

    // Sanity check to ensure that the location is valid
    if earliest_child_location.is_some() && earliest_child_location.unwrap() < location {
        panic!("Regex returned invalid location: Node cannot have a location < its children");
    }

    Some(location)
}

fn get_location_via_regexp(
    r: Regex,
    text: &str,
    parent_location: i32,
    earliest_child_location: Option<i32>,
) -> i32 {
    return find_location_via_regexp(r, text, parent_location, earliest_child_location).unwrap();
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use pg_query::NodeEnum;

    use crate::resolve_locations::derive_location;

    #[test]
    fn test_derive_location() {
        let input = "with c as (insert into contact (id) values ('id')) select * from c;";

        let insert_node = match pg_query::parse(input) {
            Ok(parsed) => Some(
                parsed
                    .protobuf
                    .nodes()
                    .iter()
                    .find(|n| match n.0.to_enum() {
                        NodeEnum::InsertStmt(_) => true,
                        _ => false,
                    })
                    .unwrap()
                    .0
                    .to_enum(),
            ),
            Err(_) => None,
        };
        let cte_location = match pg_query::parse(input) {
            Ok(parsed) => Some(
                parsed
                    .protobuf
                    .nodes()
                    .iter()
                    .find_map(|n| match n.0.to_enum() {
                        NodeEnum::CommonTableExpr(n) => Some(n.location),
                        _ => None,
                    })
                    .unwrap(),
            ),
            Err(_) => None,
        };

        let l = derive_location(
            &insert_node.unwrap(),
            input,
            cte_location.unwrap(),
            Some(23),
        );

        assert_eq!(l, Some(11));
    }
}