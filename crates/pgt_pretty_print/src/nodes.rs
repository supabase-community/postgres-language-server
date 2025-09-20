use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType, ToTokens},
};

impl ToTokens for pgt_query::NodeEnum {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match self {
            pgt_query::protobuf::node::Node::SelectStmt(stmt) => stmt.as_ref().to_tokens(e),
            pgt_query::protobuf::node::Node::ResTarget(target) => target.to_tokens(e),
            pgt_query::protobuf::node::Node::MultiAssignRef(ref_) => ref_.to_tokens(e),
            pgt_query::protobuf::node::Node::ColumnRef(col_ref) => col_ref.to_tokens(e),
            pgt_query::protobuf::node::Node::String(string) => string.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeVar(string) => string.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeSubselect(subselect) => subselect.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeFunction(func) => func.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTableSample(sample) => sample.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTableFunc(func) => func.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTableFuncCol(col) => col.to_tokens(e),
            pgt_query::protobuf::node::Node::JoinExpr(join) => join.to_tokens(e),
            pgt_query::protobuf::node::Node::FuncCall(func_call) => func_call.to_tokens(e),
            pgt_query::protobuf::node::Node::Aggref(aggref) => aggref.to_tokens(e),
            pgt_query::protobuf::node::Node::GroupingFunc(func) => func.to_tokens(e),
            pgt_query::protobuf::node::Node::WindowFunc(func) => func.to_tokens(e),
            pgt_query::protobuf::node::Node::SubscriptingRef(ref_node) => ref_node.to_tokens(e),
            pgt_query::protobuf::node::Node::CurrentOfExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::OnConflictExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::Query(query) => query.to_tokens(e),
            pgt_query::protobuf::node::Node::TargetEntry(entry) => entry.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTblRef(ref_node) => ref_node.to_tokens(e),
            pgt_query::protobuf::node::Node::FromExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::Var(var) => var.to_tokens(e),
            pgt_query::protobuf::node::Node::NextValueExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::InferenceElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::FuncExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::WindowDef(window_def) => window_def.to_tokens(e),
            pgt_query::protobuf::node::Node::SortBy(sort_by) => sort_by.to_tokens(e),
            pgt_query::protobuf::node::Node::InsertStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::List(list) => list.to_tokens(e),
            pgt_query::protobuf::node::Node::AConst(const_val) => const_val.to_tokens(e),
            pgt_query::protobuf::node::Node::DeleteStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::AIndirection(indirection) => indirection.to_tokens(e),
            pgt_query::protobuf::node::Node::UpdateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ColumnDef(def) => def.to_tokens(e),
            pgt_query::protobuf::node::Node::TypeName(type_name) => type_name.to_tokens(e),
            pgt_query::protobuf::node::Node::DropStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::TruncateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableCmd(cmd) => cmd.to_tokens(e),
            pgt_query::protobuf::node::Node::ViewStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::MergeStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::MergeWhenClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::Alias(alias) => alias.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateSchemaStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::RoleSpec(role) => role.to_tokens(e),
            pgt_query::protobuf::node::Node::GrantStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AccessPriv(privilege) => privilege.to_tokens(e),
            pgt_query::protobuf::node::Node::TransactionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VariableSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::IndexStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::IndexElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::CopyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DefElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::Boolean(b) => b.to_tokens(e),
            pgt_query::protobuf::node::Node::GrantRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDefaultPrivilegesStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VariableShowStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateTableSpaceStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropTableSpaceStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableSpaceOptionsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Float(f) => f.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableMoveAllStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateExtensionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CommentStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterExtensionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterExtensionContentsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ObjectWithArgs(obj) => obj.to_tokens(e),
            pgt_query::protobuf::node::Node::FunctionParameter(param) => param.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateFdwStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::SetOperationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateForeignServerStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterFdwStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterForeignServerStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateForeignTableStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ImportForeignSchemaStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreatePolicyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterPolicyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateAmStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateSeqStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterSeqStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Integer(i) => i.to_tokens(e),
            pgt_query::protobuf::node::Node::DefineStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateDomainStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CollateClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDomainStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Constraint(constraint) => constraint.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpClassStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpClassItem(item) => item.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpFamilyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterOpFamilyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReplicaIdentityStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::SecLabelStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterCollationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DeclareCursorStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ClosePortalStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::FetchStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AStar(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReturnStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::PlassignStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateStatsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::StatsElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterRoleSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterStatsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateFunctionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateTrigStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateEventTrigStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterEventTrigStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreatePlangStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterFunctionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DoStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::InlineCodeBlock(block) => block.to_tokens(e),
            pgt_query::protobuf::node::Node::CallStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CallContext(ctx) => ctx.to_tokens(e),
            pgt_query::protobuf::node::Node::RenameStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterObjectDependsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterObjectSchemaStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterOwnerStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterOperatorStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTypeStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterEnumStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::RuleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::NotifyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ListenStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::UnlistenStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ExecuteStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::PrepareStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropOwnedStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReassignOwnedStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTsdictionaryStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTsconfigurationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreatePublicationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterPublicationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateSubscriptionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterSubscriptionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropSubscriptionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ParamRef(param) => param.to_tokens(e),
            pgt_query::protobuf::node::Node::DeallocateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::LockStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CompositeTypeStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateEnumStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateRangeStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateTableAsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::RefreshMatViewStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::LoadStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreatedbStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropdbStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ClusterStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VacuumStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VacuumRelation(rel) => rel.to_tokens(e),
            pgt_query::protobuf::node::Node::ExplainStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDatabaseSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDatabaseRefreshCollStmt(stmt) => {
                stmt.to_tokens(e)
            }
            pgt_query::protobuf::node::Node::CheckPointStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DiscardStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ConstraintsSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReindexStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateConversionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateCastStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateTransformStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDatabaseStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterSystemStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::BitString(s) => s.to_tokens(e),
            pgt_query::protobuf::node::Node::TypeCast(tc) => tc.to_tokens(e),
            pgt_query::protobuf::node::Node::Param(p) => p.to_tokens(e),
            pgt_query::protobuf::node::Node::OpExpr(op) => op.as_ref().to_tokens(e),
            pgt_query::protobuf::node::Node::ScalarArrayOpExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::BoolExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::CaseExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::CaseWhen(when) => when.to_tokens(e),
            pgt_query::protobuf::node::Node::ArrayExpr(arr) => arr.to_tokens(e),
            pgt_query::protobuf::node::Node::RowExpr(row) => row.to_tokens(e),
            pgt_query::protobuf::node::Node::AArrayExpr(arr) => arr.to_tokens(e),
            pgt_query::protobuf::node::Node::SubLink(link) => link.to_tokens(e),
            pgt_query::protobuf::node::Node::SubPlan(plan) => plan.to_tokens(e),
            pgt_query::protobuf::node::Node::AlternativeSubPlan(plan) => plan.to_tokens(e),
            pgt_query::protobuf::node::Node::CoalesceExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::MinMaxExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::XmlExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::XmlSerialize(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::NullTest(test) => test.to_tokens(e),
            pgt_query::protobuf::node::Node::BooleanTest(test) => test.to_tokens(e),
            pgt_query::protobuf::node::Node::PublicationObjSpec(spec) => spec.to_tokens(e),
            pgt_query::protobuf::node::Node::NamedArgExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::WithClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::CommonTableExpr(cte) => cte.to_tokens(e),
            pgt_query::protobuf::node::Node::GroupingSet(gs) => gs.to_tokens(e),
            pgt_query::protobuf::node::Node::AIndices(idx) => idx.to_tokens(e),
            pgt_query::protobuf::node::Node::LockingClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::TableFunc(func) => func.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTable(table) => table.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTablePath(path) => path.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTablePathScan(scan) => scan.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTableSiblingJoin(join) => join.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonFormat(format) => format.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonIsPredicate(pred) => pred.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonReturning(ret) => ret.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonBehavior(beh) => beh.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonFuncExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonValueExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonOutput(output) => output.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonKeyValue(kv) => kv.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonObjectConstructor(ctor) => ctor.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonArrayConstructor(ctor) => ctor.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTablePathSpec(spec) => spec.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonTableColumn(col) => col.to_tokens(e),
            pgt_query::protobuf::node::Node::DistinctExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::CollateExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::NullIfExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::SqlvalueFunction(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::IntoClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::PartitionElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::PartitionSpec(spec) => spec.to_tokens(e),
            pgt_query::protobuf::node::Node::PartitionBoundSpec(spec) => spec.to_tokens(e),
            pgt_query::protobuf::node::Node::SetToDefault(def) => def.to_tokens(e),
            pgt_query::protobuf::node::Node::TableLikeClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::RelabelType(rt) => rt.to_tokens(e),
            pgt_query::protobuf::node::Node::CoerceToDomain(ctd) => ctd.to_tokens(e),
            pgt_query::protobuf::node::Node::FieldSelect(fs) => fs.to_tokens(e),
            pgt_query::protobuf::node::Node::PartitionRangeDatum(prd) => prd.to_tokens(e),
            pgt_query::protobuf::node::Node::CtesearchClause(csc) => csc.to_tokens(e),
            pgt_query::protobuf::node::Node::CtecycleClause(ccc) => ccc.to_tokens(e),
            pgt_query::protobuf::node::Node::TriggerTransition(tt) => tt.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonArgument(ja) => ja.to_tokens(e),
            pgt_query::protobuf::node::Node::PublicationTable(pt) => pt.to_tokens(e),
            pgt_query::protobuf::node::Node::CoerceViaIo(cvi) => cvi.to_tokens(e),
            pgt_query::protobuf::node::Node::FieldStore(fs) => fs.to_tokens(e),
            pgt_query::protobuf::node::Node::ArrayCoerceExpr(ace) => ace.to_tokens(e),
            pgt_query::protobuf::node::Node::ConvertRowtypeExpr(cre) => cre.to_tokens(e),
            pgt_query::protobuf::node::Node::CaseTestExpr(cte) => cte.to_tokens(e),
            pgt_query::protobuf::node::Node::CoerceToDomainValue(cdv) => cdv.to_tokens(e),
            pgt_query::protobuf::node::Node::MergeAction(ma) => ma.to_tokens(e),
            pgt_query::protobuf::node::Node::MergeSupportFunc(msf) => msf.to_tokens(e),
            pgt_query::protobuf::node::Node::SinglePartitionSpec(_) => {}
            pgt_query::protobuf::node::Node::PartitionCmd(pc) => pc.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonConstructorExpr(jce) => jce.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonParseExpr(jpe) => jpe.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonScalarExpr(jse) => jse.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonSerializeExpr(jse) => jse.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonArrayQueryConstructor(jaqc) => jaqc.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonAggConstructor(jac) => jac.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonObjectAgg(joa) => joa.to_tokens(e),
            pgt_query::protobuf::node::Node::JsonArrayAgg(jaa) => jaa.to_tokens(e),
            pgt_query::protobuf::node::Node::WindowClause(wc) => wc.to_tokens(e),
            pgt_query::protobuf::node::Node::WindowFuncRunCondition(wfrc) => wfrc.to_tokens(e),
            pgt_query::protobuf::node::Node::SortGroupClause(sgc) => sgc.to_tokens(e),
            pgt_query::protobuf::node::Node::RowMarkClause(rmc) => rmc.to_tokens(e),
            pgt_query::protobuf::node::Node::WithCheckOption(wco) => wco.to_tokens(e),
            pgt_query::protobuf::node::Node::TableSampleClause(tsc) => tsc.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTblEntry(rte) => rte.to_tokens(e),
            pgt_query::protobuf::node::Node::RtepermissionInfo(rpi) => rpi.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeTblFunction(rtf) => rtf.to_tokens(e),
            _ => {
                unimplemented!("Node type {:?} not implemented for to_tokens", self);
            }
        }
    }
}

impl ToTokens for pgt_query::Node {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(node) = &self.node {
            match node {
                pgt_query::protobuf::node::Node::SelectStmt(stmt) => stmt.as_ref().to_tokens(e),
                pgt_query::protobuf::node::Node::ResTarget(target) => target.to_tokens(e),
                pgt_query::protobuf::node::Node::MultiAssignRef(ref_) => ref_.to_tokens(e),
                pgt_query::protobuf::node::Node::ColumnRef(col_ref) => col_ref.to_tokens(e),
                pgt_query::protobuf::node::Node::String(string) => string.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeVar(string) => string.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeSubselect(subselect) => {
                    subselect.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::RangeFunction(func) => func.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTableSample(sample) => sample.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTableFunc(func) => func.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTableFuncCol(col) => col.to_tokens(e),
                pgt_query::protobuf::node::Node::JoinExpr(join) => join.to_tokens(e),
                pgt_query::protobuf::node::Node::FuncCall(func_call) => func_call.to_tokens(e),
                pgt_query::protobuf::node::Node::Aggref(aggref) => aggref.to_tokens(e),
                pgt_query::protobuf::node::Node::GroupingFunc(func) => func.to_tokens(e),
                pgt_query::protobuf::node::Node::WindowFunc(func) => func.to_tokens(e),
                pgt_query::protobuf::node::Node::SubscriptingRef(ref_node) => ref_node.to_tokens(e),
                pgt_query::protobuf::node::Node::CurrentOfExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::OnConflictExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::Query(query) => query.to_tokens(e),
                pgt_query::protobuf::node::Node::TargetEntry(entry) => entry.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTblRef(ref_node) => ref_node.to_tokens(e),
                pgt_query::protobuf::node::Node::FromExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::Var(var) => var.to_tokens(e),
                pgt_query::protobuf::node::Node::NextValueExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::InferenceElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::WindowDef(window_def) => window_def.to_tokens(e),
                pgt_query::protobuf::node::Node::SortBy(sort_by) => sort_by.to_tokens(e),
                pgt_query::protobuf::node::Node::InsertStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::List(list) => list.to_tokens(e),
                pgt_query::protobuf::node::Node::AConst(const_val) => const_val.to_tokens(e),
                pgt_query::protobuf::node::Node::DeleteStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::UpdateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ColumnDef(def) => def.to_tokens(e),
                pgt_query::protobuf::node::Node::TypeName(type_name) => type_name.to_tokens(e),
                pgt_query::protobuf::node::Node::DropStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::TruncateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableCmd(cmd) => cmd.to_tokens(e),
                pgt_query::protobuf::node::Node::ViewStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::MergeStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::MergeWhenClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::Alias(alias) => alias.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateSchemaStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::RoleSpec(role) => role.to_tokens(e),
                pgt_query::protobuf::node::Node::GrantStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AccessPriv(privilege) => privilege.to_tokens(e),
                pgt_query::protobuf::node::Node::TransactionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::VariableSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::IndexStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::IndexElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::CopyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DefElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::Boolean(b) => b.to_tokens(e),
                pgt_query::protobuf::node::Node::GrantRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDefaultPrivilegesStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::VariableShowStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateTableSpaceStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropTableSpaceStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Float(f) => f.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableMoveAllStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateExtensionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CommentStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterExtensionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterExtensionContentsStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::ObjectWithArgs(obj) => obj.to_tokens(e),
                pgt_query::protobuf::node::Node::FunctionParameter(param) => param.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateFdwStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::SetOperationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateForeignServerStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterFdwStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterForeignServerStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateForeignTableStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ImportForeignSchemaStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreatePolicyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterPolicyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateAmStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateSeqStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterSeqStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Integer(i) => i.to_tokens(e),
                pgt_query::protobuf::node::Node::DefineStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateDomainStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CollateClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDomainStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Constraint(constraint) => constraint.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpClassStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpClassItem(item) => item.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpFamilyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterOpFamilyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReplicaIdentityStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::SecLabelStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterCollationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DeclareCursorStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ClosePortalStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::FetchStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AStar(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReturnStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::PlassignStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateStatsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::StatsElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterRoleSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterStatsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateFunctionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateTrigStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateEventTrigStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterEventTrigStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreatePlangStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterFunctionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DoStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::InlineCodeBlock(block) => block.to_tokens(e),
                pgt_query::protobuf::node::Node::CallStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CallContext(ctx) => ctx.to_tokens(e),
                pgt_query::protobuf::node::Node::RenameStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterObjectDependsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterObjectSchemaStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterOwnerStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterOperatorStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTypeStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterEnumStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::RuleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::NotifyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ListenStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::UnlistenStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ExecuteStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::PrepareStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropOwnedStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReassignOwnedStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTsdictionaryStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTsconfigurationStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::CreatePublicationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterPublicationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateSubscriptionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterSubscriptionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropSubscriptionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ParamRef(param) => param.to_tokens(e),
                pgt_query::protobuf::node::Node::DeallocateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::LockStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CompositeTypeStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateEnumStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateRangeStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateTableAsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::RefreshMatViewStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::LoadStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreatedbStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropdbStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ClusterStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::VacuumStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::VacuumRelation(rel) => rel.to_tokens(e),
                pgt_query::protobuf::node::Node::ExplainStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDatabaseSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDatabaseRefreshCollStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::CheckPointStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DiscardStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ConstraintsSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReindexStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateConversionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateCastStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateTransformStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDatabaseStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterSystemStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::BitString(s) => s.to_tokens(e),
                pgt_query::protobuf::node::Node::TypeCast(tc) => tc.to_tokens(e),
                pgt_query::protobuf::node::Node::Param(p) => p.to_tokens(e),
                pgt_query::protobuf::node::Node::OpExpr(op) => op.to_tokens(e),
                pgt_query::protobuf::node::Node::ScalarArrayOpExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::BoolExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::CaseExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::CaseWhen(when) => when.to_tokens(e),
                pgt_query::protobuf::node::Node::ArrayExpr(arr) => arr.to_tokens(e),
                pgt_query::protobuf::node::Node::RowExpr(row) => row.to_tokens(e),
                pgt_query::protobuf::node::Node::AArrayExpr(arr) => arr.to_tokens(e),
                pgt_query::protobuf::node::Node::SubLink(link) => link.to_tokens(e),
                pgt_query::protobuf::node::Node::SubPlan(plan) => plan.to_tokens(e),
                pgt_query::protobuf::node::Node::AlternativeSubPlan(plan) => plan.to_tokens(e),
                pgt_query::protobuf::node::Node::CoalesceExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::MinMaxExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::XmlExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::XmlSerialize(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::NullTest(test) => test.to_tokens(e),
                pgt_query::protobuf::node::Node::BooleanTest(test) => test.to_tokens(e),
                pgt_query::protobuf::node::Node::PublicationObjSpec(spec) => spec.to_tokens(e),
                pgt_query::protobuf::node::Node::NamedArgExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::WithClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::CommonTableExpr(cte) => cte.to_tokens(e),
                pgt_query::protobuf::node::Node::GroupingSet(gs) => gs.to_tokens(e),
                pgt_query::protobuf::node::Node::AIndirection(ind) => ind.to_tokens(e),
                pgt_query::protobuf::node::Node::AIndices(idx) => idx.to_tokens(e),
                pgt_query::protobuf::node::Node::LockingClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::TableFunc(func) => func.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTable(table) => table.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTablePath(path) => path.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTablePathScan(scan) => scan.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTableSiblingJoin(join) => join.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonFormat(format) => format.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonIsPredicate(pred) => pred.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonReturning(ret) => ret.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonBehavior(beh) => beh.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonFuncExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonValueExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonOutput(output) => output.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonKeyValue(kv) => kv.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonObjectConstructor(ctor) => ctor.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonArrayConstructor(ctor) => ctor.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTablePathSpec(spec) => spec.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonTableColumn(col) => col.to_tokens(e),
                pgt_query::protobuf::node::Node::DistinctExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::CollateExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::NullIfExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::SqlvalueFunction(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::IntoClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::PartitionElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::PartitionSpec(spec) => spec.to_tokens(e),
                pgt_query::protobuf::node::Node::PartitionBoundSpec(spec) => spec.to_tokens(e),
                pgt_query::protobuf::node::Node::SetToDefault(def) => def.to_tokens(e),
                pgt_query::protobuf::node::Node::TableLikeClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::OidList(list) => list.to_tokens(e),
                pgt_query::protobuf::node::Node::IntList(list) => list.to_tokens(e),
                pgt_query::protobuf::node::Node::OnConflictClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::InferClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::RelabelType(rt) => rt.to_tokens(e),
                pgt_query::protobuf::node::Node::CoerceToDomain(ctd) => ctd.to_tokens(e),
                pgt_query::protobuf::node::Node::FieldSelect(fs) => fs.to_tokens(e),
                pgt_query::protobuf::node::Node::PartitionRangeDatum(prd) => prd.to_tokens(e),
                pgt_query::protobuf::node::Node::CtesearchClause(csc) => csc.to_tokens(e),
                pgt_query::protobuf::node::Node::CtecycleClause(ccc) => ccc.to_tokens(e),
                pgt_query::protobuf::node::Node::TriggerTransition(tt) => tt.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonArgument(ja) => ja.to_tokens(e),
                pgt_query::protobuf::node::Node::PublicationTable(pt) => pt.to_tokens(e),
                pgt_query::protobuf::node::Node::CoerceViaIo(cvi) => cvi.to_tokens(e),
                pgt_query::protobuf::node::Node::FieldStore(fs) => fs.to_tokens(e),
                pgt_query::protobuf::node::Node::ArrayCoerceExpr(ace) => ace.to_tokens(e),
                pgt_query::protobuf::node::Node::ConvertRowtypeExpr(cre) => cre.to_tokens(e),
                pgt_query::protobuf::node::Node::CaseTestExpr(cte) => cte.to_tokens(e),
                pgt_query::protobuf::node::Node::CoerceToDomainValue(cdv) => cdv.to_tokens(e),
                pgt_query::protobuf::node::Node::MergeAction(ma) => ma.to_tokens(e),
                pgt_query::protobuf::node::Node::MergeSupportFunc(msf) => msf.to_tokens(e),
                pgt_query::protobuf::node::Node::SinglePartitionSpec(_) => {}
                pgt_query::protobuf::node::Node::PartitionCmd(pc) => pc.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonConstructorExpr(jce) => jce.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonParseExpr(jpe) => jpe.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonScalarExpr(jse) => jse.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonSerializeExpr(jse) => jse.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonArrayQueryConstructor(jaqc) => {
                    jaqc.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::JsonAggConstructor(jac) => jac.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonObjectAgg(joa) => joa.to_tokens(e),
                pgt_query::protobuf::node::Node::JsonArrayAgg(jaa) => jaa.to_tokens(e),
                pgt_query::protobuf::node::Node::WindowClause(wc) => wc.to_tokens(e),
                pgt_query::protobuf::node::Node::WindowFuncRunCondition(wfrc) => wfrc.to_tokens(e),
                pgt_query::protobuf::node::Node::SortGroupClause(sgc) => sgc.to_tokens(e),
                pgt_query::protobuf::node::Node::RowMarkClause(rmc) => rmc.to_tokens(e),
                pgt_query::protobuf::node::Node::WithCheckOption(wco) => wco.to_tokens(e),
                pgt_query::protobuf::node::Node::TableSampleClause(tsc) => tsc.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTblEntry(rte) => rte.to_tokens(e),
                pgt_query::protobuf::node::Node::RtepermissionInfo(rpi) => rpi.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeTblFunction(rtf) => rtf.to_tokens(e),
                _ => {
                    unimplemented!("Node type {:?} not implemented for to_tokens", node);
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::SelectStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SelectStmt);

        if let Some(ref with_clause) = self.with_clause {
            with_clause.to_tokens(e);
            e.line(LineType::SoftOrSpace);
        }

        use pgt_query::protobuf::SetOperation;
        let is_set_operation = matches!(
            self.op(),
            SetOperation::SetopUnion | SetOperation::SetopIntersect | SetOperation::SetopExcept
        );

        if is_set_operation {
            if let Some(ref larg) = self.larg {
                larg.as_ref().to_tokens(e);
            }

            match self.op() {
                SetOperation::SetopUnion => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::UNION_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                SetOperation::SetopIntersect => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::INTERSECT_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                SetOperation::SetopExcept => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::EXCEPT_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                _ => {}
            }

            if let Some(ref rarg) = self.rarg {
                rarg.as_ref().to_tokens(e);
            }
        } else if !self.values_lists.is_empty() {
            e.token(TokenKind::VALUES_KW);
            e.space();
            for (i, values_list) in self.values_lists.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                values_list.to_tokens(e);
            }
        } else {
            e.token(TokenKind::SELECT_KW);

            if !self.target_list.is_empty() {
                e.indent_start();
                e.line(LineType::SoftOrSpace);

                for (i, target) in self.target_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    target.to_tokens(e);
                }
                e.indent_end();
            }

            if let Some(ref into_clause) = self.into_clause {
                into_clause.to_tokens(e);
            }

            if !self.from_clause.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::FROM_KW);
                e.line(LineType::SoftOrSpace);

                e.indent_start();
                for (i, from) in self.from_clause.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    from.to_tokens(e);
                }
                e.indent_end();
            }

            if let Some(ref where_clause) = self.where_clause {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WHERE_KW);
                e.space();
                where_clause.to_tokens(e);
            }

            if !self.group_clause.is_empty() {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::GROUP_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.indent_start();
                for (i, group) in self.group_clause.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                    }
                    e.line(LineType::SoftOrSpace);
                    group.to_tokens(e);
                }
                e.indent_end();
            }

            if !self.locking_clause.is_empty() {
                for clause in &self.locking_clause {
                    e.space();
                    clause.to_tokens(e);
                }
            }

            if let Some(ref having) = self.having_clause {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::HAVING_KW);
                e.space();
                having.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ResTarget {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ResTarget);

        if e.is_within_group(GroupKind::UpdateStmt) {
            if !self.name.is_empty() {
                e.token(TokenKind::IDENT(self.name.clone()));
                for d in &self.indirection {
                    e.token(TokenKind::DOT);
                    d.to_tokens(e);
                }
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
            }
            if let Some(ref val) = self.val {
                val.to_tokens(e);
            }
        } else if let Some(ref val) = self.val {
            val.to_tokens(e);
            if !self.name.is_empty() {
                e.space();
                e.token(TokenKind::AS_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
        } else if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::MultiAssignRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref source) = self.source {
            source.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ColumnRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ColumnRef);

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            field.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::String {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.sval.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::RangeVar {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RangeVar);

        if !self.schemaname.is_empty() {
            e.token(TokenKind::IDENT(self.schemaname.clone()));
            e.token(TokenKind::DOT);
        }

        e.token(TokenKind::IDENT(self.relname.clone()));

        if let Some(ref alias) = self.alias {
            e.space();
            alias.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JoinExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::JoinType;

        e.group_start(GroupKind::JoinExpr);

        if let Some(ref larg) = self.larg {
            larg.to_tokens(e);
        }

        e.line(LineType::SoftOrSpace);

        if self.is_natural {
            e.token(TokenKind::NATURAL_KW);
            e.space();
        }

        match self.jointype() {
            JoinType::JoinInner => {
                e.token(TokenKind::INNER_KW);
                e.space();
                e.token(TokenKind::JOIN_KW);
            }
            JoinType::JoinLeft => {
                e.token(TokenKind::LEFT_KW);
                e.space();
                e.token(TokenKind::JOIN_KW);
            }
            JoinType::JoinFull => {
                e.token(TokenKind::FULL_KW);
                e.space();
                e.token(TokenKind::JOIN_KW);
            }
            JoinType::JoinRight => {
                e.token(TokenKind::RIGHT_KW);
                e.space();
                e.token(TokenKind::JOIN_KW);
            }
            _ => {
                e.token(TokenKind::JOIN_KW);
            }
        }

        e.indent_start();
        e.line(LineType::SoftOrSpace);

        if let Some(ref rarg) = self.rarg {
            rarg.to_tokens(e);
        }

        e.indent_end();

        if !self.using_clause.is_empty() {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.using_clause.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        } else if let Some(ref quals) = self.quals {
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            quals.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FuncCall {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::FuncCall);

        for (i, name) in self.funcname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
            }
            name.to_tokens(e);
        }

        e.token(TokenKind::L_PAREN);

        if self.agg_star {
            e.token(TokenKind::IDENT("*".to_string()));
        } else if !self.args.is_empty() || !self.agg_order.is_empty() {
            e.group_start(GroupKind::FuncCall);
            e.line(LineType::Soft);
            e.indent_start();

            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                arg.to_tokens(e);
            }

            if !self.agg_order.is_empty() {
                e.space();
                e.token(TokenKind::ORDER_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                for (i, agg_order) in self.agg_order.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    agg_order.to_tokens(e);
                }
            }

            e.indent_end();
            e.line(LineType::Soft);
            e.group_end();
        }

        e.token(TokenKind::R_PAREN);

        if let Some(ref over) = self.over {
            e.space();
            e.token(TokenKind::OVER_KW);
            e.space();
            over.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::WindowDef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::WindowDef);

        e.token(TokenKind::L_PAREN);

        let mut need_space = false;

        if !self.partition_clause.is_empty() {
            e.token(TokenKind::PARTITION_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();

            for (i, partition) in self.partition_clause.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                partition.to_tokens(e);
            }
            need_space = true;
        }

        if !self.order_clause.is_empty() {
            if need_space {
                e.space();
            }
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();

            for (i, order) in self.order_clause.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                order.to_tokens(e);
            }
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SortBy {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::{SortByDir, SortByNulls};

        e.group_start(GroupKind::SortBy);

        if let Some(ref node) = self.node {
            node.to_tokens(e);
        }

        match self.sortby_dir() {
            SortByDir::SortbyAsc => {
                e.space();
                e.token(TokenKind::ASC_KW);
            }
            SortByDir::SortbyDesc => {
                e.space();
                e.token(TokenKind::DESC_KW);
            }
            SortByDir::SortbyDefault | SortByDir::Undefined => {}
            SortByDir::SortbyUsing => todo!(),
        }

        match self.sortby_nulls() {
            SortByNulls::SortbyNullsFirst => {
                e.space();
                e.token(TokenKind::NULLS_KW);
                e.space();
                e.token(TokenKind::FIRST_KW);
            }
            SortByNulls::SortbyNullsLast => {
                e.space();
                e.token(TokenKind::NULLS_KW);
                e.space();
                e.token(TokenKind::LAST_KW);
            }
            SortByNulls::SortbyNullsDefault | SortByNulls::Undefined => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::InsertStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::InsertStmt);

        e.token(TokenKind::INSERT_KW);
        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.cols.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.cols.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref select_stmt) = self.select_stmt {
            e.space();
            select_stmt.to_tokens(e);
        }

        if let Some(ref on_conflict_clause) = self.on_conflict_clause {
            e.space();
            on_conflict_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::List {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::List);

        if e.is_within_group(GroupKind::DropStmt) {
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                item.to_tokens(e);
            }
        } else if e.is_within_group(GroupKind::CommentStmt) {
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                item.to_tokens(e);
            }
        } else if e.is_within_group(GroupKind::AlterTsconfigurationStmt) {
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
        } else if e.is_within_group(GroupKind::AExpr) {
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
        } else {
            e.token(TokenKind::L_PAREN);
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::OidList {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::OidList);
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            item.to_tokens(e);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::IntList {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IntList);
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            item.to_tokens(e);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AConst {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref val) = self.val {
            match val {
                pgt_query::protobuf::a_const::Val::Ival(ival) => {
                    e.token(TokenKind::IDENT(ival.ival.to_string()));
                }
                pgt_query::protobuf::a_const::Val::Fval(fval) => {
                    e.token(TokenKind::IDENT(fval.fval.clone()));
                }
                pgt_query::protobuf::a_const::Val::Boolval(boolval) => {
                    let val_str = if boolval.boolval { "TRUE" } else { "FALSE" };
                    e.token(TokenKind::IDENT(val_str.to_string()));
                }
                pgt_query::protobuf::a_const::Val::Sval(sval) => {
                    e.token(TokenKind::STRING(format!("'{}'", sval.sval)));
                }
                pgt_query::protobuf::a_const::Val::Bsval(bsval) => {
                    bsval.to_tokens(e);
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::DeleteStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DeleteStmt);

        e.token(TokenKind::DELETE_KW);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::AExprKind;

        e.group_start(GroupKind::AExpr);

        if e.is_within_group(GroupKind::TypeCast) {
            e.token(TokenKind::L_PAREN);
        }

        match self.kind() {
            AExprKind::AexprOpAny | AExprKind::AexprOpAll => {
                if let Some(ref lexpr) = self.lexpr {
                    lexpr.to_tokens(e);
                }

                if !self.name.is_empty() {
                    e.space();
                    for name in &self.name {
                        name.to_tokens(e);
                    }
                    e.space();
                }

                if self.kind == AExprKind::AexprOpAny as i32 {
                    e.token(TokenKind::ANY_KW);
                } else {
                    e.token(TokenKind::ALL_KW);
                }
                e.token(TokenKind::L_PAREN);

                if let Some(ref rexpr) = self.rexpr {
                    rexpr.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            AExprKind::AexprIn => {
                if let Some(ref lexpr) = self.lexpr {
                    lexpr.to_tokens(e);
                }

                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::L_PAREN);

                if let Some(ref rexpr) = self.rexpr {
                    rexpr.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            AExprKind::AexprDistinct => {
                if let Some(ref lexpr) = self.lexpr {
                    lexpr.to_tokens(e);
                }

                e.space();
                e.token(TokenKind::IS_KW);
                e.space();
                e.token(TokenKind::DISTINCT_KW);
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();

                if let Some(ref rexpr) = self.rexpr {
                    rexpr.to_tokens(e);
                }
            }
            AExprKind::AexprNullif => {
                e.token(TokenKind::NULLIF_KW);
                e.token(TokenKind::L_PAREN);

                if let Some(ref lexpr) = self.lexpr {
                    lexpr.to_tokens(e);
                }

                e.token(TokenKind::COMMA);
                e.space();

                if let Some(ref rexpr) = self.rexpr {
                    rexpr.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            _ => {
                if let Some(ref lexpr) = self.lexpr {
                    lexpr.to_tokens(e);
                }

                if !self.name.is_empty() {
                    e.space();
                    for name in &self.name {
                        name.to_tokens(e);
                    }
                    e.space();
                }

                if let Some(ref rexpr) = self.rexpr {
                    rexpr.to_tokens(e);
                }
            }
        }

        if e.is_within_group(GroupKind::TypeCast) {
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::UpdateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::UpdateStmt);

        e.token(TokenKind::UPDATE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.target_list.is_empty() {
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            for (i, target) in self.target_list.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                target.to_tokens(e);
            }
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if let Some(ref partbound) = self.partbound {
            e.space();
            e.token(TokenKind::PARTITION_KW);
            e.space();
            e.token(TokenKind::OF_KW);
            e.space();
            if !self.inh_relations.is_empty() {
                if let Some(ref parent) = self.inh_relations.first() {
                    parent.to_tokens(e);
                }
            }
            partbound.to_tokens(e);
        }

        if !self.table_elts.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, elt) in self.table_elts.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                elt.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref partspec) = self.partspec {
            partspec.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ColumnDef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ColumnDef);

        e.token(TokenKind::IDENT(self.colname.clone()));

        if let Some(ref type_name) = self.type_name {
            e.space();
            type_name.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::TypeName {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TypeName);

        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        for (i, obj) in self.objects.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            obj.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RowCompareExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RowCompareExpr);

        e.token(TokenKind::L_PAREN);
        for (i, arg) in self.largs.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        e.space();

        use pgt_query::protobuf::RowCompareType;
        match self.rctype() {
            RowCompareType::RowcompareLt => e.token(TokenKind::IDENT("<".to_string())),
            RowCompareType::RowcompareLe => e.token(TokenKind::IDENT("<=".to_string())),
            RowCompareType::RowcompareEq => e.token(TokenKind::IDENT("=".to_string())),
            RowCompareType::RowcompareGe => e.token(TokenKind::IDENT(">=".to_string())),
            RowCompareType::RowcompareGt => e.token(TokenKind::IDENT(">".to_string())),
            RowCompareType::RowcompareNe => e.token(TokenKind::IDENT("<>".to_string())),
            RowCompareType::Undefined => todo!(),
        }

        e.space();

        e.token(TokenKind::L_PAREN);
        for (i, arg) in self.rargs.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::TruncateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TruncateStmt);

        e.token(TokenKind::TRUNCATE_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        for (i, rel) in self.relations.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            rel.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        for (i, cmd) in self.cmds.iter().enumerate() {
            if i == 0 {
                e.space();
            } else {
                e.token(TokenKind::COMMA);
                e.space();
            }
            cmd.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableCmd {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableCmd);

        use pgt_query::protobuf::AlterTableType;

        match self.subtype() {
            AlterTableType::AtAddColumn => {
                e.token(TokenKind::ADD_KW);
                e.space();
                e.token(TokenKind::COLUMN_KW);
                if let Some(ref def) = self.def {
                    e.space();
                    def.to_tokens(e);
                }
            }
            AlterTableType::AtDropColumn => {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::COLUMN_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            AlterTableType::AtReplicaIdentity => {
                e.token(TokenKind::REPLICA_KW);
                e.space();
                e.token(TokenKind::IDENTITY_KW);
                e.space();
                if let Some(ref def) = self.def {
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &def.node {
                        e.token(TokenKind::IDENT(s.sval.clone()));
                    } else {
                        def.to_tokens(e);
                    }
                }
            }
            AlterTableType::AtChangeOwner => {
                e.token(TokenKind::OWNER_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                if let Some(ref newowner) = self.newowner {
                    newowner.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ViewStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ViewStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::VIEW_KW);
        e.space();

        if let Some(ref view) = self.view {
            view.to_tokens(e);
        }

        if let Some(ref query) = self.query {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::MergeStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::MergeStmt);

        e.token(TokenKind::MERGE_KW);
        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();

        if let Some(ref source_relation) = self.source_relation {
            source_relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref join_condition) = self.join_condition {
            join_condition.to_tokens(e);
        }

        for when_clause in &self.merge_when_clauses {
            e.line(LineType::SoftOrSpace);
            when_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::MergeWhenClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::MergeWhenClause);

        e.token(TokenKind::WHEN_KW);
        e.space();

        use pgt_query::protobuf::{CmdType, MergeMatchKind};

        match self.match_kind() {
            MergeMatchKind::MergeWhenMatched => {
                e.token(TokenKind::MATCHED_KW);
            }
            MergeMatchKind::MergeWhenNotMatchedByTarget => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::MATCHED_KW);
            }
            MergeMatchKind::MergeWhenNotMatchedBySource => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::MATCHED_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.token(TokenKind::SOURCE_KW);
            }
            _ => {}
        }

        if let Some(ref condition) = self.condition {
            e.space();
            e.token(TokenKind::AND_KW);
            e.space();
            condition.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::THEN_KW);
        e.space();

        match self.command_type() {
            CmdType::CmdInsert => {
                e.token(TokenKind::INSERT_KW);
                if !self.target_list.is_empty() {
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, target) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        target.to_tokens(e);
                    }
                    e.token(TokenKind::R_PAREN);
                }
                if !self.values.is_empty() {
                    e.space();
                    e.token(TokenKind::VALUES_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, val) in self.values.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        val.to_tokens(e);
                    }
                    e.token(TokenKind::R_PAREN);
                }
            }
            CmdType::CmdUpdate => {
                e.group_start(GroupKind::UpdateStmt);
                e.token(TokenKind::UPDATE_KW);
                if !self.target_list.is_empty() {
                    e.space();
                    e.token(TokenKind::SET_KW);
                    e.space();
                    for (i, target) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        target.to_tokens(e);
                    }
                }
                e.group_end();
            }
            CmdType::CmdDelete => {
                e.token(TokenKind::DELETE_KW);
            }
            CmdType::CmdNothing => {
                e.token(TokenKind::DO_KW);
                e.space();
                e.token(TokenKind::NOTHING_KW);
            }
            _ => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RangeSubselect {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RangeSubselect);

        if self.lateral {
            e.token(TokenKind::LATERAL_KW);
            e.space();
        }

        e.token(TokenKind::L_PAREN);
        if let Some(ref subquery) = self.subquery {
            subquery.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        if let Some(ref alias) = self.alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            alias.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RangeFunction {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RangeFunction);

        if self.lateral {
            e.token(TokenKind::LATERAL_KW);
            e.space();
        }

        if self.is_rowsfrom {
            e.token(TokenKind::ROWS_KW);
            e.space();
            e.token(TokenKind::FROM_KW);
            e.token(TokenKind::L_PAREN);
        }

        for (i, func) in self.functions.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }

            if let Some(pgt_query::protobuf::node::Node::List(list)) = &func.node {
                if let Some(first_item) = list.items.first() {
                    first_item.to_tokens(e);
                }
            }
        }

        if self.is_rowsfrom {
            e.token(TokenKind::R_PAREN);
        }

        if self.ordinality {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::ORDINALITY_KW);
        }

        if let Some(ref alias) = self.alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            alias.to_tokens(e);

            if !self.coldeflist.is_empty() {
                e.token(TokenKind::L_PAREN);
                for (i, col) in self.coldeflist.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    col.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Alias {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.aliasname.clone()));
        if !self.colnames.is_empty() {
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.colnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::CreateSchemaStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateSchemaStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.schemaname.is_empty() {
            e.space();
            e.token(TokenKind::IDENT(self.schemaname.clone()));
        }

        if let Some(ref authrole) = self.authrole {
            e.space();
            e.token(TokenKind::AUTHORIZATION_KW);
            e.space();
            authrole.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RoleSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::RoleSpecType;
        match self.roletype() {
            RoleSpecType::RolespecCstring => {
                if !self.rolename.is_empty() {
                    e.token(TokenKind::IDENT(self.rolename.clone()));
                }
            }
            RoleSpecType::RolespecCurrentRole => {
                e.token(TokenKind::CURRENT_ROLE_KW);
            }
            RoleSpecType::RolespecCurrentUser => {
                e.token(TokenKind::CURRENT_USER_KW);
            }
            RoleSpecType::RolespecSessionUser => {
                e.token(TokenKind::SESSION_USER_KW);
            }
            RoleSpecType::RolespecPublic => {
                e.token(TokenKind::IDENT("PUBLIC".to_string()));
            }
            _ => {
                if !self.rolename.is_empty() {
                    e.token(TokenKind::IDENT(self.rolename.clone()));
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::GrantStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::GrantStmt);

        if self.is_grant {
            e.token(TokenKind::GRANT_KW);
        } else {
            e.token(TokenKind::REVOKE_KW);
        }
        e.space();

        if !self.privileges.is_empty() {
            for (i, privilege) in self.privileges.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                privilege.to_tokens(e);
            }
        } else {
            e.token(TokenKind::ALL_KW);
            e.space();
            e.token(TokenKind::PRIVILEGES_KW);
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.objtype() {
            ObjectType::ObjectTable => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::TABLES_KW);
                } else {
                    e.token(TokenKind::TABLE_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            ObjectType::ObjectSchema => {
                e.token(TokenKind::SCHEMA_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            ObjectType::ObjectDatabase => {
                e.token(TokenKind::DATABASE_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            ObjectType::ObjectFunction => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::FUNCTIONS_KW);
                } else {
                    e.token(TokenKind::FUNCTION_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::PROCEDURE_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            ObjectType::ObjectSequence => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::SEQUENCES_KW);
                } else {
                    e.token(TokenKind::SEQUENCE_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            _ => {}
        }

        for (i, obj) in self.objects.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            obj.to_tokens(e);
        }

        if self.is_grant {
            e.space();
            e.token(TokenKind::TO_KW);
        } else {
            e.space();
            e.token(TokenKind::FROM_KW);
        }
        e.space();

        for (i, grantee) in self.grantees.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            grantee.to_tokens(e);
        }

        if self.grant_option && self.is_grant {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::GRANT_KW);
            e.space();
            e.token(TokenKind::OPTION_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AccessPriv {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.priv_name.to_uppercase()));

        if !self.cols.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.cols.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::TransactionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TransactionStmt);

        use pgt_query::protobuf::TransactionStmtKind;
        match self.kind() {
            TransactionStmtKind::TransStmtBegin => {
                e.token(TokenKind::BEGIN_KW);
            }
            TransactionStmtKind::TransStmtStart => {
                e.token(TokenKind::START_KW);
                e.space();
                e.token(TokenKind::TRANSACTION_KW);
            }
            TransactionStmtKind::TransStmtCommit => {
                e.token(TokenKind::COMMIT_KW);
            }
            TransactionStmtKind::TransStmtRollback => {
                e.token(TokenKind::ROLLBACK_KW);
            }
            TransactionStmtKind::TransStmtSavepoint => {
                e.token(TokenKind::SAVEPOINT_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtRelease => {
                e.token(TokenKind::RELEASE_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::SAVEPOINT_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtRollbackTo => {
                e.token(TokenKind::ROLLBACK_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::SAVEPOINT_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtPrepare => {
                e.token(TokenKind::PREPARE_KW);
                e.space();
                e.token(TokenKind::TRANSACTION_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            TransactionStmtKind::TransStmtCommitPrepared => {
                e.token(TokenKind::COMMIT_KW);
                e.space();
                e.token(TokenKind::PREPARED_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            TransactionStmtKind::TransStmtRollbackPrepared => {
                e.token(TokenKind::ROLLBACK_KW);
                e.space();
                e.token(TokenKind::PREPARED_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VariableSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::VariableSetStmt);

        use pgt_query::protobuf::VariableSetKind;
        match self.kind() {
            VariableSetKind::VarSetValue => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));

                if !self.args.is_empty() {
                    e.space();
                    e.token(TokenKind::TO_KW);
                    e.space();

                    for (i, arg) in self.args.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        arg.to_tokens(e);
                    }
                }
            }
            VariableSetKind::VarSetDefault => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
            }
            VariableSetKind::VarSetCurrent => {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
                e.token(TokenKind::CURRENT_KW);
            }
            VariableSetKind::VarSetMulti => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
            }
            VariableSetKind::VarReset => {
                e.token(TokenKind::RESET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
            }
            VariableSetKind::VarResetAll => {
                e.token(TokenKind::RESET_KW);
                e.space();
                e.token(TokenKind::ALL_KW);
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::IndexStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IndexStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.unique {
            e.token(TokenKind::UNIQUE_KW);
            e.space();
        }

        if self.concurrent {
            e.token(TokenKind::CONCURRENTLY_KW);
            e.space();
        }

        e.token(TokenKind::INDEX_KW);
        e.space();

        if self.if_not_exists {
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
            e.space();
        }

        if !self.idxname.is_empty() {
            e.token(TokenKind::IDENT(self.idxname.clone()));
            e.space();
        }

        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.index_params.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, param) in self.index_params.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                param.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::IndexElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IndexElem);

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        } else if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        if !self.opclass.is_empty() {
            e.space();
            for (i, opclass) in self.opclass.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                opclass.to_tokens(e);
            }
        }

        use pgt_query::protobuf::SortByDir;
        if self.ordering != SortByDir::SortbyDefault as i32 {
            e.space();
            match self.ordering() {
                SortByDir::SortbyAsc => e.token(TokenKind::ASC_KW),
                SortByDir::SortbyDesc => e.token(TokenKind::DESC_KW),
                _ => {}
            }
        }

        use pgt_query::protobuf::SortByNulls;
        if self.nulls_ordering != SortByNulls::SortbyNullsDefault as i32 {
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            match self.nulls_ordering() {
                SortByNulls::SortbyNullsFirst => e.token(TokenKind::FIRST_KW),
                SortByNulls::SortbyNullsLast => e.token(TokenKind::LAST_KW),
                _ => {}
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CopyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CopyStmt);

        e.token(TokenKind::COPY_KW);
        e.space();

        if self.is_from {
            if !self.filename.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.filename)));
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
            }

            if let Some(ref relation) = self.relation {
                relation.to_tokens(e);
            }
        } else {
            if let Some(ref relation) = self.relation {
                relation.to_tokens(e);
            }

            if !self.attlist.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, attr) in self.attlist.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    attr.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }

            if let Some(ref query) = self.query {
                e.space();
                e.token(TokenKind::L_PAREN);
                query.to_tokens(e);
                e.token(TokenKind::R_PAREN);
            }

            e.space();
            e.token(TokenKind::TO_KW);
            e.space();

            if !self.filename.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.filename)));
            } else {
                e.token(TokenKind::STDOUT_KW);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();

            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DefElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DefElem);

        if self.defname == "format" {
            if let Some(ref arg) = self.arg {
                if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                    e.token(TokenKind::IDENT(s.sval.to_uppercase()));
                }
            }
        } else if self.defname == "header" {
            e.token(TokenKind::HEADER_KW);
        } else if self.defname == "delimiter" {
            e.token(TokenKind::DELIMITER_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "quote" {
            e.token(TokenKind::QUOTE_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "escape" {
            e.token(TokenKind::ESCAPE_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "null" {
            e.token(TokenKind::NULL_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "encoding" {
            e.token(TokenKind::ENCODING_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else {
            if e.is_within_group(GroupKind::AlterTableSpaceOptionsStmt) {
                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::IDENT("=".to_string()));
                    e.space();
                    arg.to_tokens(e);
                }
            } else if e.is_within_group(GroupKind::AlterExtensionStmt)
                && self.defname == "new_version"
            {
                e.token(TokenKind::UPDATE_KW);
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::TO_KW);
                    e.space();
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                        e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                    } else {
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateRoleStmt)
                || e.is_within_group(GroupKind::AlterRoleStmt)
            {
                if self.defname == "canlogin" {
                    e.token(TokenKind::IDENT("LOGIN".to_string()));
                } else if self.defname == "password" {
                    e.token(TokenKind::PASSWORD_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                        } else {
                            arg.to_tokens(e);
                        }
                    }
                } else if self.defname == "superuser" {
                    e.token(TokenKind::IDENT("SUPERUSER".to_string()));
                } else if self.defname == "createdb" {
                    e.token(TokenKind::IDENT("CREATEDB".to_string()));
                } else if self.defname == "createrole" {
                    e.token(TokenKind::IDENT("CREATEROLE".to_string()));
                } else if self.defname == "inherit" {
                    e.token(TokenKind::INHERIT_KW);
                } else if self.defname == "replication" {
                    e.token(TokenKind::IDENT("REPLICATION".to_string()));
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::AlterSeqStmt) {
                if self.defname == "restart" {
                    e.token(TokenKind::RESTART_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        e.token(TokenKind::WITH_KW);
                        e.space();
                        arg.to_tokens(e);
                    }
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateForeignTableStmt)
                || e.is_within_group(GroupKind::CreateForeignServerStmt)
                || e.is_within_group(GroupKind::AlterForeignServerStmt)
                || e.is_within_group(GroupKind::CreateUserMappingStmt)
                || e.is_within_group(GroupKind::AlterUserMappingStmt)
                || e.is_within_group(GroupKind::AlterFdwStmt)
            {
                use pgt_query::protobuf::DefElemAction;

                match self.defaction() {
                    DefElemAction::DefelemAdd => {
                        e.token(TokenKind::ADD_KW);
                        e.space();
                    }
                    DefElemAction::DefelemSet => {
                        e.token(TokenKind::SET_KW);
                        e.space();
                    }
                    DefElemAction::DefelemDrop => {
                        e.token(TokenKind::DROP_KW);
                        e.space();
                    }
                    _ => {}
                }

                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                        e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                    } else {
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::DoStmt) {
                if self.defname == "language" {
                    e.token(TokenKind::LANGUAGE_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                } else if self.defname == "as" {
                    if let Some(ref arg) = self.arg {
                        e.space();
                        if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                        } else {
                            arg.to_tokens(e);
                        }
                    }
                }
            } else if e.is_within_group(GroupKind::CreateFunctionStmt) {
                if self.defname == "as" {
                    e.token(TokenKind::AS_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        if let Some(pgt_query::protobuf::node::Node::List(list)) = &arg.node {
                            // Handle function body as list
                            if let Some(first) = list.items.first() {
                                if let Some(pgt_query::protobuf::node::Node::String(s)) =
                                    &first.node
                                {
                                    e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                                } else {
                                    first.to_tokens(e);
                                }
                            }
                        } else if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                        } else {
                            arg.to_tokens(e);
                        }
                    }
                } else if self.defname == "language" {
                    e.token(TokenKind::LANGUAGE_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::AlterFunctionStmt) {
                if self.defname == "volatility" && self.arg.is_some() {
                    if let Some(ref arg) = self.arg {
                        if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            match s.sval.as_str() {
                                "i" => e.token(TokenKind::IMMUTABLE_KW),
                                "s" => e.token(TokenKind::STABLE_KW),
                                "v" => e.token(TokenKind::VOLATILE_KW),
                                _ => e.token(TokenKind::IDENT(s.sval.to_uppercase())),
                            }
                        }
                    }
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateRangeStmt) {
                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::IDENT("=".to_string()));
                    e.space();
                    arg.to_tokens(e);
                }
            } else if e.is_within_group(GroupKind::AlterTsdictionaryStmt) {
                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::IDENT("=".to_string()));
                    e.space();
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                        e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                    } else {
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::DefineStmt) {
                if self.defname == "from" {
                    e.token(TokenKind::FROM_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        if let Some(pgt_query::protobuf::node::Node::List(list)) = &arg.node {
                            // Handle list of collation names
                            if let Some(first) = list.items.first() {
                                if let Some(pgt_query::protobuf::node::Node::String(s)) =
                                    &first.node
                                {
                                    e.token(TokenKind::IDENT(format!("\"{}\"", s.sval)));
                                } else {
                                    first.to_tokens(e);
                                }
                            }
                        } else if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            e.token(TokenKind::IDENT(format!("\"{}\"", s.sval)));
                        } else {
                            arg.to_tokens(e);
                        }
                    }
                } else {
                    e.token(TokenKind::IDENT(self.defname.clone()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else {
                e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    arg.to_tokens(e);
                }
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Boolean {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::Boolean);

        if self.boolval {
            e.token(TokenKind::TRUE_KW);
        } else {
            e.token(TokenKind::FALSE_KW);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::GrantRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::GrantRoleStmt);

        if self.is_grant {
            e.token(TokenKind::GRANT_KW);
        } else {
            e.token(TokenKind::REVOKE_KW);
        }
        e.space();

        for (i, role) in self.granted_roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            role.to_tokens(e);
        }

        e.space();
        if self.is_grant {
            e.token(TokenKind::TO_KW);
        } else {
            e.token(TokenKind::FROM_KW);
        }
        e.space();

        for (i, grantee) in self.grantee_roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            grantee.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterDefaultPrivilegesStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDefaultPrivilegesStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        e.token(TokenKind::PRIVILEGES_KW);

        if let Some(ref action) = self.action {
            e.space();
            action.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VariableShowStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::VariableShowStmt);

        e.token(TokenKind::SHOW_KW);
        e.space();
        e.token(TokenKind::IDENT(self.name.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateTableSpaceStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateTableSpaceStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if let Some(ref owner) = self.owner {
            e.space();
            e.token(TokenKind::OWNER_KW);
            e.space();
            owner.to_tokens(e);
        }

        if !self.location.is_empty() {
            e.space();
            e.token(TokenKind::LOCATION_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.location)));
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropTableSpaceStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropTableSpaceStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableSpaceOptionsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableSpaceOptionsStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if self.is_reset {
            e.space();
            e.token(TokenKind::RESET_KW);
        } else {
            e.space();
            e.token(TokenKind::SET_KW);
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Float {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.fval.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableMoveAllStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableMoveAllStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();

        match self.objtype {
            x if x == pgt_query::protobuf::ObjectType::ObjectTable as i32 => {
                e.token(TokenKind::TABLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectIndex as i32 => {
                e.token(TokenKind::INDEX_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectMatview as i32 => {
                e.token(TokenKind::MATERIALIZED_KW);
                e.space();
                e.token(TokenKind::VIEW_KW);
            }
            _ => {}
        }

        e.space();
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.orig_tablespacename.clone()));

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::OWNED_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.new_tablespacename.clone()));

        if self.nowait {
            e.space();
            e.token(TokenKind::NOWAIT_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateExtensionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateExtensionStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));

        if !self.options.is_empty() {
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CommentStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CommentStmt);

        e.token(TokenKind::COMMENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        match self.objtype {
            x if x == pgt_query::protobuf::ObjectType::ObjectTable as i32 => {
                e.token(TokenKind::TABLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectColumn as i32 => {
                e.token(TokenKind::COLUMN_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectFunction as i32 => {
                e.token(TokenKind::FUNCTION_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectIndex as i32 => {
                e.token(TokenKind::INDEX_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectView as i32 => {
                e.token(TokenKind::VIEW_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectSequence as i32 => {
                e.token(TokenKind::SEQUENCE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectSchema as i32 => {
                e.token(TokenKind::SCHEMA_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectDatabase as i32 => {
                e.token(TokenKind::DATABASE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectRole as i32 => {
                e.token(TokenKind::ROLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectType as i32 => {
                e.token(TokenKind::TYPE_KW);
            }
            _ => {}
        }

        e.space();
        if let Some(ref object) = self.object {
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();

        if self.comment.is_empty() {
            e.token(TokenKind::NULL_KW);
        } else {
            e.token(TokenKind::STRING(format!("'{}'", self.comment)));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterExtensionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterExtensionStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));

        if !self.options.is_empty() {
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterExtensionContentsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterExtensionContentsStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));
        e.space();

        if self.action > 0 {
            e.token(TokenKind::ADD_KW);
        } else {
            e.token(TokenKind::DROP_KW);
        }
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.objtype() {
            ObjectType::ObjectFunction => {
                e.token(TokenKind::FUNCTION_KW);
            }
            ObjectType::ObjectTable => {
                e.token(TokenKind::TABLE_KW);
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::TYPE_KW);
            }
            ObjectType::ObjectOperator => {
                e.token(TokenKind::OPERATOR_KW);
            }
            _ => {}
        }

        if let Some(ref object) = self.object {
            e.space();
            object.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ObjectWithArgs {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ObjectWithArgs);

        for (i, name) in self.objname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if !self.objargs.is_empty() || !self.objfuncargs.is_empty() || self.args_unspecified {
            e.token(TokenKind::L_PAREN);

            let args_to_print = if !self.objfuncargs.is_empty() {
                &self.objfuncargs
            } else {
                &self.objargs
            };

            for (i, arg) in args_to_print.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FunctionParameter {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::FunctionParameter);

        if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
            e.space();
        }

        if let Some(ref arg_type) = self.arg_type {
            arg_type.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateFdwStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateFdwStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.func_options.is_empty() {
            e.space();
            e.token(TokenKind::HANDLER_KW);
            e.space();
            for (i, option) in self.func_options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateRoleStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        use pgt_query::protobuf::RoleStmtType;
        match self.stmt_type() {
            RoleStmtType::RolestmtRole => e.token(TokenKind::ROLE_KW),
            RoleStmtType::RolestmtUser => e.token(TokenKind::USER_KW),
            RoleStmtType::RolestmtGroup => e.token(TokenKind::GROUP_KW),
            _ => e.token(TokenKind::ROLE_KW),
        }

        e.space();
        e.token(TokenKind::IDENT(self.role.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SetOperationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SetOperationStmt);

        if let Some(ref larg) = self.larg {
            larg.to_tokens(e);
        }

        use pgt_query::protobuf::SetOperation;
        match self.op() {
            SetOperation::SetopUnion => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::UNION_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            SetOperation::SetopIntersect => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::INTERSECT_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            SetOperation::SetopExcept => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::EXCEPT_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            _ => {}
        }

        if let Some(ref rarg) = self.rarg {
            rarg.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateForeignServerStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateForeignServerStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.servertype.is_empty() {
            e.space();
            e.token(TokenKind::TYPE_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.servertype)));
        }

        if !self.version.is_empty() {
            e.space();
            e.token(TokenKind::VERSION_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.version)));
        }

        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterFdwStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterFdwStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.func_options.is_empty() {
            e.space();
            e.token(TokenKind::HANDLER_KW);
            e.space();
            for (i, option) in self.func_options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterForeignServerStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterForeignServerStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if self.has_version {
            e.space();
            e.token(TokenKind::VERSION_KW);
            e.space();
            if !self.version.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.version)));
            } else {
                e.token(TokenKind::NULL_KW);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateForeignTableStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateForeignTableStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref base_stmt) = self.base_stmt {
            if let Some(ref relation) = base_stmt.relation {
                relation.to_tokens(e);
            }

            if !base_stmt.table_elts.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::SoftOrSpace);
                for (i, elt) in base_stmt.table_elts.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    elt.to_tokens(e);
                }
                e.indent_end();
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::R_PAREN);
            }
        }

        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateUserMappingStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        } else {
            e.token(TokenKind::CURRENT_USER_KW);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterUserMappingStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        } else {
            e.token(TokenKind::CURRENT_USER_KW);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropUserMappingStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ImportForeignSchemaStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ImportForeignSchemaStmt);

        e.token(TokenKind::IMPORT_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
        e.token(TokenKind::IDENT(self.remote_schema.clone()));

        use pgt_query::protobuf::ImportForeignSchemaType;
        match self.list_type() {
            ImportForeignSchemaType::FdwImportSchemaLimitTo => {
                e.space();
                e.token(TokenKind::LIMIT_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, table) in self.table_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    table.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            ImportForeignSchemaType::FdwImportSchemaExcept => {
                e.space();
                e.token(TokenKind::EXCEPT_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, table) in self.table_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    table.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            _ => {}
        }

        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.server_name.clone()));

        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();
        e.token(TokenKind::IDENT(self.local_schema.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreatePolicyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreatePolicyStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::POLICY_KW);
        e.space();
        e.token(TokenKind::IDENT(self.policy_name.clone()));

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref table) = self.table {
            table.to_tokens(e);
        }

        if !self.permissive {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT("RESTRICTIVE".to_string()));
        }

        if !self.cmd_name.is_empty() && self.cmd_name != "all" {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            match self.cmd_name.as_str() {
                "select" => e.token(TokenKind::SELECT_KW),
                "insert" => e.token(TokenKind::INSERT_KW),
                "update" => e.token(TokenKind::UPDATE_KW),
                "delete" => e.token(TokenKind::DELETE_KW),
                _ => e.token(TokenKind::IDENT(self.cmd_name.clone())),
            }
        } else if self.cmd_name == "all" {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
        }

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if let Some(ref qual) = self.qual {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            qual.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref with_check) = self.with_check {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            with_check.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterPolicyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterPolicyStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::POLICY_KW);
        e.space();
        e.token(TokenKind::IDENT(self.policy_name.clone()));

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref table) = self.table {
            table.to_tokens(e);
        }

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if let Some(ref qual) = self.qual {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            qual.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref with_check) = self.with_check {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            with_check.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateAmStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateAmStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::ACCESS_KW);
        e.space();
        e.token(TokenKind::METHOD_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();
        match self.amtype.as_str() {
            "t" => e.token(TokenKind::TABLE_KW),
            "i" => e.token(TokenKind::INDEX_KW),
            _ => e.token(TokenKind::IDENT(self.amtype.clone())),
        }

        e.space();
        e.token(TokenKind::HANDLER_KW);
        e.space();
        for (i, handler) in self.handler_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            handler.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateSeqStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateSeqStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SEQUENCE_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if let Some(ref sequence) = self.sequence {
            e.space();
            sequence.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for option in &self.options {
                option.to_tokens(e);
                e.space();
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterSeqStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterSeqStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SEQUENCE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if let Some(ref sequence) = self.sequence {
            e.space();
            sequence.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Integer {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.ival.to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::DefineStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DefineStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.kind() {
            ObjectType::ObjectAggregate => {
                e.token(TokenKind::AGGREGATE_KW);
            }
            ObjectType::ObjectOperator => {
                e.token(TokenKind::OPERATOR_KW);
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::TYPE_KW);
            }
            ObjectType::ObjectCollation => {
                e.token(TokenKind::COLLATION_KW);
            }
            _ => todo!(),
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.args.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if !self.definition.is_empty() {
            e.space();

            // Special handling for CREATE COLLATION ... FROM
            if self.kind() == ObjectType::ObjectCollation
                && self.definition.len() == 1
                && self.definition[0].node.as_ref().map_or(false, |n| {
                    matches!(n, pgt_query::protobuf::node::Node::DefElem(elem) if elem.defname == "from")
                })
            {
                // For CREATE COLLATION ... FROM, don't use parentheses
                self.definition[0].to_tokens(e);
            } else {
                e.token(TokenKind::L_PAREN);
                for (i, def) in self.definition.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    def.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateDomainStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateDomainStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::DOMAIN_KW);
        e.space();

        for (i, name) in self.domainname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if let Some(ref type_name) = self.type_name {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            type_name.to_tokens(e);
        }

        if let Some(ref coll_clause) = self.coll_clause {
            e.space();
            coll_clause.to_tokens(e);
        }

        for constraint in &self.constraints {
            e.space();
            constraint.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CollateClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(arg) = &self.arg {
            arg.to_tokens(e);
            e.space();
        }

        e.token(TokenKind::COLLATE_KW);
        e.space();

        for (i, name) in self.collname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            e.token(TokenKind::IDENT('"'.to_string()));
            name.to_tokens(e);
            e.token(TokenKind::IDENT('"'.to_string()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterDomainStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDomainStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DOMAIN_KW);

        if self.missing_ok && self.subtype == "T" {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        for (i, name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        match self.subtype.as_str() {
            "T" => {
                e.space();
                e.token(TokenKind::TYPE_KW);
            }
            "N" => {
                e.space();
                if let Some(ref def) = self.def {
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &def.node {
                        if s.sval == "NOT NULL" {
                            e.token(TokenKind::SET_KW);
                            e.space();
                            e.token(TokenKind::NOT_KW);
                            e.space();
                            e.token(TokenKind::NULL_KW);
                        } else if s.sval == "NULL" {
                            e.token(TokenKind::DROP_KW);
                            e.space();
                            e.token(TokenKind::NOT_KW);
                            e.space();
                            e.token(TokenKind::NULL_KW);
                        }
                    }
                }
            }
            "O" => {
                e.space();
                e.token(TokenKind::OWNER_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                if let Some(ref def) = self.def {
                    def.to_tokens(e);
                }
            }
            "R" => {
                e.space();
                e.token(TokenKind::RENAME_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "S" => {
                e.space();
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::SCHEMA_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "A" => {
                e.space();
                e.token(TokenKind::ADD_KW);
                e.space();
                if let Some(ref def) = self.def {
                    def.to_tokens(e);
                }
            }
            "D" | "X" => {
                e.space();
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::CONSTRAINT_KW);
                if self.missing_ok {
                    e.space();
                    e.token(TokenKind::IF_KW);
                    e.space();
                    e.token(TokenKind::EXISTS_KW);
                }
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "V" => {
                e.space();
                e.token(TokenKind::VALIDATE_KW);
                e.space();
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            _ => panic!("Unknown ALTER DOMAIN subtype: {}", self.subtype),
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Constraint {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::Constraint);

        if !self.conname.is_empty() {
            e.token(TokenKind::CONSTRAINT_KW);
            e.space();
            e.token(TokenKind::IDENT(self.conname.clone()));
            e.space();
        }

        use pgt_query::protobuf::ConstrType;
        match self.contype() {
            ConstrType::ConstrCheck => {
                e.token(TokenKind::CHECK_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                if let Some(ref raw_expr) = self.raw_expr {
                    raw_expr.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            ConstrType::ConstrPrimary => {
                e.token(TokenKind::PRIMARY_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
            }
            ConstrType::ConstrUnique => {
                e.token(TokenKind::UNIQUE_KW);
            }
            ConstrType::ConstrForeign => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
            }
            ConstrType::ConstrNotnull => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::NULL_KW);
            }
            ConstrType::ConstrDefault => {
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                if let Some(ref raw_expr) = self.raw_expr {
                    raw_expr.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        if self.is_no_inherit {
            e.space();
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::INHERIT_KW);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpClassStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpClassStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::CLASS_KW);
        e.space();

        for (i, name) in self.opclassname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if self.is_default {
            e.space();
            e.token(TokenKind::DEFAULT_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        if let Some(ref datatype) = self.datatype {
            datatype.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        if !self.opfamilyname.is_empty() {
            e.space();
            e.token(TokenKind::FAMILY_KW);
            e.space();
            for (i, name) in self.opfamilyname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.items.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpClassItem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpClassItem);

        match self.itemtype {
            1 => {
                e.token(TokenKind::OPERATOR_KW);
                e.space();
                e.token(TokenKind::IDENT(self.number.to_string()));
                e.space();
                if let Some(ref name) = self.name {
                    name.to_tokens(e);
                }
            }
            2 => {
                e.token(TokenKind::FUNCTION_KW);
                e.space();
                e.token(TokenKind::IDENT(self.number.to_string()));
                e.space();
                if let Some(ref name) = self.name {
                    name.to_tokens(e);
                }
            }
            3 => {
                e.token(TokenKind::STORAGE_KW);
                e.space();
                if let Some(ref storedtype) = self.storedtype {
                    storedtype.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        if !self.order_family.is_empty() {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            for (i, name) in self.order_family.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpFamilyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpFamilyStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::FAMILY_KW);
        e.space();

        for (i, name) in self.opfamilyname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterOpFamilyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterOpFamilyStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::FAMILY_KW);
        e.space();

        for (i, name) in self.opfamilyname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));
        e.space();

        if self.is_drop {
            e.token(TokenKind::DROP_KW);
        } else {
            e.token(TokenKind::ADD_KW);
        }
        e.space();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            item.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ReplicaIdentityStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match self.identity_type.as_str() {
            "d" => e.token(TokenKind::DEFAULT_KW),
            "n" => e.token(TokenKind::NOTHING_KW),
            "f" => e.token(TokenKind::FULL_KW),
            "i" => {
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            _ => panic!("Unknown replica identity type: {}", self.identity_type),
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterCollationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterCollationStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::COLLATION_KW);
        e.space();

        for (i, name) in self.collname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::REFRESH_KW);
        e.space();
        e.token(TokenKind::VERSION_KW);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DeclareCursorStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DeclareCursorStmt);

        e.token(TokenKind::DECLARE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.portalname.clone()));
        e.space();
        e.token(TokenKind::CURSOR_KW);

        // TODO: Handle cursor options (options field)

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref query) = self.query {
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ClosePortalStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ClosePortalStmt);

        e.token(TokenKind::CLOSE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.portalname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SecLabelStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SecLabelStmt);

        e.token(TokenKind::SECURITY_KW);
        e.space();
        e.token(TokenKind::LABEL_KW);

        if !self.provider.is_empty() {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::IDENT(self.provider.clone()));
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.objtype() {
            ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
            ObjectType::ObjectColumn => e.token(TokenKind::COLUMN_KW),
            ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
            ObjectType::ObjectRole => e.token(TokenKind::ROLE_KW),
            ObjectType::ObjectDatabase => e.token(TokenKind::DATABASE_KW),
            ObjectType::ObjectTablespace => e.token(TokenKind::TABLESPACE_KW),
            ObjectType::ObjectSchema => e.token(TokenKind::SCHEMA_KW),
            ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
            ObjectType::ObjectDomain => e.token(TokenKind::DOMAIN_KW),
            ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
            ObjectType::ObjectLanguage => e.token(TokenKind::LANGUAGE_KW),
            ObjectType::ObjectLargeobject => {
                e.token(TokenKind::LARGE_KW);
                e.space();
                e.token(TokenKind::OBJECT_KW);
            }
            ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
            ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
            _ => todo!(),
        }

        if let Some(ref object) = self.object {
            e.space();
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.label)));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AStar {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("*".to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::ReturnStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ReturnStmt);

        e.token(TokenKind::RETURN_KW);

        if let Some(ref returnval) = self.returnval {
            e.space();
            returnval.as_ref().to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FetchStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::FetchDirection;

        e.group_start(GroupKind::FetchStmt);

        if self.ismove {
            e.token(TokenKind::MOVE_KW);
        } else {
            e.token(TokenKind::FETCH_KW);
        }

        match self.direction() {
            FetchDirection::FetchForward => {
                if self.how_many == 0 {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                } else if self.how_many > 1 {
                    e.space();
                    e.token(TokenKind::FORWARD_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.how_many.to_string()));
                } else {
                    e.space();
                    e.token(TokenKind::NEXT_KW);
                }
            }
            FetchDirection::FetchBackward => {
                if self.how_many == 0 {
                    e.space();
                    e.token(TokenKind::BACKWARD_KW);
                    e.space();
                    e.token(TokenKind::ALL_KW);
                } else if self.how_many > 1 {
                    e.space();
                    e.token(TokenKind::BACKWARD_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.how_many.to_string()));
                } else {
                    e.space();
                    e.token(TokenKind::PRIOR_KW);
                }
            }
            FetchDirection::FetchAbsolute => {
                e.space();
                e.token(TokenKind::ABSOLUTE_KW);
                e.space();
                e.token(TokenKind::IDENT(self.how_many.to_string()));
            }
            FetchDirection::FetchRelative => {
                e.space();
                e.token(TokenKind::RELATIVE_KW);
                e.space();
                e.token(TokenKind::IDENT(self.how_many.to_string()));
            }
            _ => {}
        }

        if !self.portalname.is_empty() {
            e.space();
            e.token(TokenKind::FROM_KW);
            e.space();
            e.token(TokenKind::IDENT(self.portalname.clone()));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateStatsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateStatsStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::STATISTICS_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.stat_types.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, stat_type) in self.stat_types.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                stat_type.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if !self.exprs.is_empty() {
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            for (i, expr) in self.exprs.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                expr.to_tokens(e);
            }
        }

        if !self.relations.is_empty() {
            e.space();
            e.token(TokenKind::FROM_KW);
            e.space();
            for (i, relation) in self.relations.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                relation.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::StatsElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        } else {
            e.token(TokenKind::IDENT(self.name.clone()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterRoleStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if let Some(ref role) = self.role {
            e.space();
            role.to_tokens(e);
        }

        if self.action == 1 {
            e.space();
            e.token(TokenKind::SET_KW);
        } else if self.action == -1 {
            e.space();
            e.token(TokenKind::RESET_KW);
        } else if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterRoleSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterRoleSetStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if let Some(ref role) = self.role {
            e.space();
            role.to_tokens(e);
        }

        if !self.database.is_empty() {
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::DATABASE_KW);
            e.space();
            e.token(TokenKind::IDENT(self.database.clone()));
        }

        if let Some(ref setstmt) = self.setstmt {
            e.space();
            setstmt.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropRoleStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.roles.is_empty() {
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterStatsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterStatsStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::STATISTICS_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if let Some(ref target) = self.stxstattarget {
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::STATISTICS_KW);
            e.space();
            target.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateFunctionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateFunctionStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.replace {
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
            e.space();
        }

        if self.is_procedure {
            e.token(TokenKind::PROCEDURE_KW);
        } else {
            e.token(TokenKind::FUNCTION_KW);
        }

        if !self.funcname.is_empty() {
            e.space();
            for (i, name) in self.funcname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.space();
        e.token(TokenKind::L_PAREN);
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            param.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        if let Some(ref return_type) = self.return_type {
            e.space();
            e.token(TokenKind::RETURNS_KW);
            e.space();
            return_type.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for option in &self.options {
                option.to_tokens(e);
                e.space();
            }
        }

        if let Some(ref sql_body) = self.sql_body {
            e.space();
            sql_body.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateTrigStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateTrigStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.replace {
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
            e.space();
        }

        e.token(TokenKind::TRIGGER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.trigname.clone()));
        e.line(LineType::SoftOrSpace);

        e.token(TokenKind::AFTER_KW);
        e.space();
        e.token(TokenKind::INSERT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }
        e.line(LineType::SoftOrSpace);

        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::EACH_KW);
        e.space();
        e.token(TokenKind::ROW_KW);
        e.line(LineType::SoftOrSpace);

        e.token(TokenKind::EXECUTE_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
        e.space();

        if !self.funcname.is_empty() {
            for (i, name) in self.funcname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateEventTrigStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateEventTrigStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::EVENT_KW);
        e.space();
        e.token(TokenKind::TRIGGER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.trigname.clone()));
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::IDENT(self.eventname.clone()));
        e.line(LineType::SoftOrSpace);

        if !self.whenclause.is_empty() {
            e.token(TokenKind::WHEN_KW);
            e.space();
            for (i, clause) in self.whenclause.iter().enumerate() {
                if i > 0 {
                    e.space();
                    e.token(TokenKind::AND_KW);
                    e.space();
                }
                clause.to_tokens(e);
            }
            e.line(LineType::SoftOrSpace);
        }

        e.token(TokenKind::EXECUTE_KW);
        e.space();
        e.token(TokenKind::FUNCTION_KW);
        e.space();

        if !self.funcname.is_empty() {
            for (i, name) in self.funcname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterEventTrigStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterEventTrigStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::EVENT_KW);
        e.space();
        e.token(TokenKind::TRIGGER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.trigname.clone()));
        e.space();

        match self.tgenabled.as_str() {
            "O" => e.token(TokenKind::ENABLE_KW),
            "D" => e.token(TokenKind::DISABLE_KW),
            "R" => {
                e.token(TokenKind::ENABLE_KW);
                e.space();
                e.token(TokenKind::REPLICA_KW);
            }
            "A" => {
                e.token(TokenKind::ENABLE_KW);
                e.space();
                e.token(TokenKind::ALWAYS_KW);
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreatePLangStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreatePlangStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.replace {
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
            e.space();
        }

        if self.pltrusted {
            e.token(TokenKind::TRUSTED_KW);
            e.space();
        }

        e.token(TokenKind::LANGUAGE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.plname.clone()));

        if !self.plhandler.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::HANDLER_KW);
            e.space();
            for (i, handler) in self.plhandler.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                handler.to_tokens(e);
            }
        }

        if !self.plinline.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::INLINE_KW);
            e.space();
            for (i, inline) in self.plinline.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                inline.to_tokens(e);
            }
        }

        if !self.plvalidator.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::VALIDATOR_KW);
            e.space();
            for (i, validator) in self.plvalidator.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                validator.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterFunctionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterFunctionStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();

        match self.objtype {
            x if x == pgt_query::protobuf::ObjectType::ObjectFunction as i32 => {
                e.token(TokenKind::FUNCTION_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectProcedure as i32 => {
                e.token(TokenKind::PROCEDURE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectRoutine as i32 => {
                e.token(TokenKind::ROUTINE_KW);
            }
            _ => {
                e.token(TokenKind::FUNCTION_KW);
            }
        }
        e.space();

        if let Some(ref func) = self.func {
            func.to_tokens(e);
        }

        for action in &self.actions {
            e.line(LineType::SoftOrSpace);
            action.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::InlineCodeBlock {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::InlineCodeBlock);

        e.token(TokenKind::STRING(self.source_text.clone()));

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DoStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DoStmt);
        e.token(TokenKind::DO_KW);

        if !self.args.is_empty() {
            e.space();
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.line(LineType::SoftOrSpace);
                }
                arg.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CallStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CallStmt);

        e.token(TokenKind::CALL_KW);
        e.space();

        if let Some(ref funccall) = self.funccall {
            funccall.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RenameStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RenameStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.rename_type() {
            ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
            ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
            ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
            ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
            _ => todo!(),
        }
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::RENAME_KW);
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        e.token(TokenKind::IDENT(self.newname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterObjectDependsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterObjectDependsStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.object_type() {
            ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
            ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
            ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
            ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
            _ => todo!(),
        }
        e.space();

        if let Some(ref object) = self.object {
            object.to_tokens(e);
        }

        e.space();
        if self.remove {
            e.token(TokenKind::NO_KW);
            e.space();
        }
        e.token(TokenKind::DEPENDS_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);
        e.space();

        if let Some(ref extname) = self.extname {
            extname.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterObjectSchemaStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterObjectSchemaStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.object_type() {
            ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
            ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
            ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
            ObjectType::ObjectMatview => {
                e.token(TokenKind::MATERIALIZED_KW);
                e.space();
                e.token(TokenKind::VIEW_KW);
            }
            ObjectType::ObjectForeignTable => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::TABLE_KW);
            }
            _ => todo!(),
        }

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        } else if let Some(ref object) = self.object {
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
        e.token(TokenKind::IDENT(self.newschema.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterOwnerStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterOwnerStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match self.object_type() {
            ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
            ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
            ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
            ObjectType::ObjectMatview => {
                e.token(TokenKind::MATERIALIZED_KW);
                e.space();
                e.token(TokenKind::VIEW_KW);
            }
            ObjectType::ObjectForeignTable => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::TABLE_KW);
            }
            ObjectType::ObjectDatabase => e.token(TokenKind::DATABASE_KW),
            ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
            ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
            ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
            ObjectType::ObjectSchema => e.token(TokenKind::SCHEMA_KW),
            ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
            ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
            _ => todo!(),
        }
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        } else if let Some(ref object) = self.object {
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::OWNER_KW);
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();

        if let Some(ref newowner) = self.newowner {
            newowner.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterOperatorStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterOperatorStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();

        if let Some(ref opername) = self.opername {
            opername.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTypeStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTypeStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        for (i, type_name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            type_name.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterEnumStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterEnumStmt);
        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        for (i, type_name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            type_name.to_tokens(e);
        }

        e.space();
        if !self.old_val.is_empty() {
            e.token(TokenKind::RENAME_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.old_val)));
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.new_val)));
        } else {
            e.token(TokenKind::ADD_KW);
            e.space();
            e.token(TokenKind::VALUE_KW);
            if self.skip_if_new_val_exists {
                e.space();
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
            }
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.new_val)));
            if !self.new_val_neighbor.is_empty() {
                e.space();
                if self.new_val_is_after {
                    e.token(TokenKind::AFTER_KW);
                } else {
                    e.token(TokenKind::BEFORE_KW);
                }
                e.space();
                e.token(TokenKind::STRING(format!("'{}'", self.new_val_neighbor)));
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RuleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RuleStmt);
        e.token(TokenKind::CREATE_KW);
        if self.replace {
            e.space();
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
        }
        e.space();
        e.token(TokenKind::RULE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.rulename.clone()));
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        use pgt_query::protobuf::CmdType;
        match self.event() {
            CmdType::CmdSelect => e.token(TokenKind::SELECT_KW),
            CmdType::CmdInsert => e.token(TokenKind::INSERT_KW),
            CmdType::CmdUpdate => e.token(TokenKind::UPDATE_KW),
            CmdType::CmdDelete => e.token(TokenKind::DELETE_KW),
            _ => {}
        }

        e.space();
        e.token(TokenKind::TO_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::DO_KW);
        e.space();

        if self.instead {
            e.token(TokenKind::INSTEAD_KW);
            e.space();
        }

        if self.actions.len() == 1 && !self.actions[0].node.is_none() {
            self.actions[0].to_tokens(e);
        } else if self.actions.len() > 1 {
            e.token(TokenKind::L_PAREN);
            for (i, action) in self.actions.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::SEMICOLON);
                    e.space();
                }
                action.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        } else {
            e.token(TokenKind::NOTHING_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::NotifyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::NotifyStmt);
        e.token(TokenKind::NOTIFY_KW);
        e.space();
        e.token(TokenKind::IDENT(self.conditionname.clone()));

        if !self.payload.is_empty() {
            e.token(TokenKind::COMMA);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.payload)));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ListenStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ListenStmt);
        e.token(TokenKind::LISTEN_KW);
        e.space();
        e.token(TokenKind::IDENT(self.conditionname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::UnlistenStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::UnlistenStmt);
        e.token(TokenKind::UNLISTEN_KW);
        e.space();
        e.token(TokenKind::IDENT(self.conditionname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ExecuteStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ExecuteStmt);
        e.token(TokenKind::EXECUTE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.name.clone()));

        if !self.params.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, param) in self.params.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                param.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PrepareStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PrepareStmt);
        e.token(TokenKind::PREPARE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.name.clone()));

        if !self.argtypes.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, argtype) in self.argtypes.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                argtype.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();

        if let Some(query) = &self.query {
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ParamRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ParamRef);
        e.token(TokenKind::IDENT(format!("${}", self.number)));
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DeallocateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DeallocateStmt);
        e.token(TokenKind::DEALLOCATE_KW);
        e.space();

        if self.isall {
            e.token(TokenKind::ALL_KW);
        } else {
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::LockStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::LockStmt);
        e.token(TokenKind::LOCK_KW);
        e.space();

        if !self.relations.is_empty() {
            e.token(TokenKind::TABLE_KW);
            e.space();

            for (i, relation) in self.relations.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                relation.to_tokens(e);
            }
        }

        match self.mode {
            1 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::ACCESS_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            2 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::ROW_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            3 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::ROW_KW);
                e.space();
                e.token(TokenKind::EXCLUSIVE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            4 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
                e.space();
                e.token(TokenKind::UPDATE_KW);
                e.space();
                e.token(TokenKind::EXCLUSIVE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            5 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            6 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
                e.space();
                e.token(TokenKind::ROW_KW);
                e.space();
                e.token(TokenKind::EXCLUSIVE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            7 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::EXCLUSIVE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            8 => {
                e.space();
                e.token(TokenKind::IN_KW);
                e.space();
                e.token(TokenKind::ACCESS_KW);
                e.space();
                e.token(TokenKind::EXCLUSIVE_KW);
                e.space();
                e.token(TokenKind::MODE_KW);
            }
            _ => {}
        }

        if self.nowait {
            e.space();
            e.token(TokenKind::NOWAIT_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CompositeTypeStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CompositeTypeStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        if let Some(ref typevar) = self.typevar {
            typevar.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        for (i, col) in self.coldeflist.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            col.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateEnumStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateEnumStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        for (i, name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::ENUM_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        for (i, val) in self.vals.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            if let Some(pgt_query::protobuf::node::Node::String(s)) = &val.node {
                e.token(TokenKind::STRING(format!("'{}'", s.sval)));
            } else {
                val.to_tokens(e);
            }
        }

        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PlAssignStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PlassignStmt);

        e.token(TokenKind::IDENT(self.name.clone()));

        for indirection in &self.indirection {
            indirection.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IDENT(":=".to_string()));
        e.space();

        if let Some(ref val) = self.val {
            val.as_ref().to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateRangeStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateRangeStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        for (i, name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::RANGE_KW);

        if !self.params.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);

            for (i, param) in self.params.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                param.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateTableAsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateTableAsStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if let Some(ref into) = self.into {
            if self.objtype == pgt_query::protobuf::ObjectType::Undefined as i32 {
                if into.on_commit != pgt_query::protobuf::OnCommitAction::OncommitNoop as i32 {
                    e.token(TokenKind::TEMP_KW);
                    e.space();
                }
            } else if self.objtype == pgt_query::protobuf::ObjectType::ObjectTable as i32 {
                e.token(TokenKind::TABLE_KW);
                e.space();
            } else if self.objtype == pgt_query::protobuf::ObjectType::ObjectMatview as i32 {
                e.token(TokenKind::MATERIALIZED_KW);
                e.space();
                e.token(TokenKind::VIEW_KW);
                e.space();
            }

            if self.if_not_exists {
                e.token(TokenKind::IF_KW);
                e.space();
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::EXISTS_KW);
                e.space();
            }

            if let Some(ref rel) = into.rel {
                rel.to_tokens(e);
            }

            if !into.col_names.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, col) in into.col_names.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    col.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
        } else {
            e.token(TokenKind::TABLE_KW);
            e.space();
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();

        if let Some(ref query) = self.query {
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RefreshMatViewStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RefreshMatViewStmt);

        e.token(TokenKind::REFRESH_KW);
        e.space();
        e.token(TokenKind::MATERIALIZED_KW);
        e.space();
        e.token(TokenKind::VIEW_KW);
        e.space();

        if self.concurrent {
            e.token(TokenKind::CONCURRENTLY_KW);
            e.space();
        }

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if self.skip_data {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::DATA_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::LoadStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::LoadStmt);

        e.token(TokenKind::LOAD_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.filename)));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreatedbStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreatedbStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.dbname.clone()));

        if !self.options.is_empty() {
            todo!();
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropdbStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropdbStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();

        if self.missing_ok {
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
            e.space();
        }

        e.token(TokenKind::IDENT(self.dbname.clone()));

        if !self.options.is_empty() {
            todo!();
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ClusterStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ClusterStmt);

        e.token(TokenKind::CLUSTER_KW);

        if !self.params.is_empty() {
            todo!();
        }

        if let Some(ref relation) = self.relation {
            e.space();
            relation.to_tokens(e);
        }

        if !self.indexname.is_empty() {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::IDENT(self.indexname.clone()));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VacuumStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::VacuumStmt);

        if self.is_vacuumcmd {
            e.token(TokenKind::VACUUM_KW);
        } else {
            e.token(TokenKind::ANALYZE_KW);
        }

        if !self.options.is_empty() {
            todo!();
        }

        if !self.rels.is_empty() {
            e.space();
            let mut first = true;
            for rel in &self.rels {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                first = false;
                rel.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VacuumRelation {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.va_cols.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            let mut first = true;
            for col in &self.va_cols {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                first = false;
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ExplainStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ExplainStmt);

        e.token(TokenKind::EXPLAIN_KW);

        if !self.options.is_empty() {
            todo!();
        }

        if let Some(ref query) = self.query {
            e.space();
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterDatabaseSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDatabaseSetStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.dbname.clone()));
        e.space();

        if let Some(ref setstmt) = self.setstmt {
            setstmt.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterDatabaseRefreshCollStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDatabaseRefreshCollStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.dbname.clone()));
        e.space();
        e.token(TokenKind::REFRESH_KW);
        e.space();
        e.token(TokenKind::COLLATION_KW);
        e.space();
        e.token(TokenKind::VERSION_KW);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CheckPointStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CheckPointStmt);

        e.token(TokenKind::CHECKPOINT_KW);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DiscardStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::DiscardMode;

        e.group_start(GroupKind::DiscardStmt);

        e.token(TokenKind::DISCARD_KW);
        e.space();

        match self.target() {
            DiscardMode::DiscardAll => e.token(TokenKind::ALL_KW),
            DiscardMode::DiscardPlans => e.token(TokenKind::PLANS_KW),
            DiscardMode::DiscardSequences => e.token(TokenKind::SEQUENCES_KW),
            DiscardMode::DiscardTemp => e.token(TokenKind::TEMP_KW),
            _ => todo!(),
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ConstraintsSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ConstraintsSetStmt);

        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::CONSTRAINTS_KW);
        e.space();

        if self.constraints.is_empty() {
            e.token(TokenKind::ALL_KW);
        } else {
            let mut first = true;
            for constraint in &self.constraints {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                first = false;
                constraint.to_tokens(e);
            }
        }

        e.space();
        if self.deferred {
            e.token(TokenKind::DEFERRED_KW);
        } else {
            e.token(TokenKind::IMMEDIATE_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ReindexStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::ReindexObjectType;

        e.group_start(GroupKind::ReindexStmt);

        e.token(TokenKind::REINDEX_KW);

        if !self.params.is_empty() {
            todo!();
        }

        e.space();
        match self.kind() {
            ReindexObjectType::ReindexObjectIndex => e.token(TokenKind::INDEX_KW),
            ReindexObjectType::ReindexObjectTable => e.token(TokenKind::TABLE_KW),
            ReindexObjectType::ReindexObjectSchema => e.token(TokenKind::SCHEMA_KW),
            ReindexObjectType::ReindexObjectSystem => e.token(TokenKind::SYSTEM_KW),
            ReindexObjectType::ReindexObjectDatabase => e.token(TokenKind::DATABASE_KW),
            _ => todo!(),
        }

        e.space();
        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        } else if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterDatabaseStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDatabaseStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DATABASE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.dbname.clone()));

        if !self.options.is_empty() {
            e.space();
            let mut first = true;
            for option in &self.options {
                if !first {
                    e.space();
                }
                first = false;
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterSystemStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterSystemStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SYSTEM_KW);
        e.space();

        if let Some(ref setstmt) = self.setstmt {
            setstmt.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::BitString {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if self.bsval.starts_with("b'") || self.bsval.starts_with("B'") {
            e.token(TokenKind::STRING(self.bsval.to_uppercase()));
        } else if self.bsval.starts_with('b') || self.bsval.starts_with('B') {
            let digits = &self.bsval[1..];
            e.token(TokenKind::STRING(format!("B'{}'", digits)));
        } else {
            e.token(TokenKind::STRING(format!("B'{}'", self.bsval)));
        }
    }
}

impl ToTokens for pgt_query::protobuf::TypeCast {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TypeCast);

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        e.token(TokenKind::DOUBLE_COLON);

        if let Some(ref type_name) = self.type_name {
            type_name.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Param {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::DOLLAR);
        e.token(TokenKind::IDENT(self.paramid.to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::OpExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        // OpExpr is handled by delegating to its arguments
        // The operator information is stored in opno/opresulttype but we
        // reconstruct from the AST context
        if self.args.len() >= 2 {
            // Binary operator
            self.args[0].to_tokens(e);
            e.space();
            e.token(TokenKind::IDENT("+".to_string())); // Default to +, actual op would need lookup
            e.space();
            self.args[1].to_tokens(e);
        } else if self.args.len() == 1 {
            // Unary operator
            e.token(TokenKind::IDENT("-".to_string())); // Default to -, actual op would need lookup
            self.args[0].to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ScalarArrayOpExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ScalarArrayOpExpr);

        if self.args.len() >= 2 {
            self.args[0].to_tokens(e);
            e.space();
            e.token(TokenKind::IDENT("=".to_string()));
            e.space();
            if self.use_or {
                e.token(TokenKind::ANY_KW);
            } else {
                e.token(TokenKind::ALL_KW);
            }
            e.token(TokenKind::L_PAREN);
            self.args[1].to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::BoolExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::BoolExprType;

        match self.boolop() {
            BoolExprType::AndExpr => {
                // Add indentation for multi-line AND expressions in JOIN ON clauses
                let needs_indent = e.is_within_group(GroupKind::JoinExpr);
                if needs_indent && self.args.len() > 1 {
                    e.indent_start();
                }

                for (i, arg) in self.args.iter().enumerate() {
                    if i > 0 {
                        e.indent_start();
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::AND_KW);
                        e.space();
                        arg.to_tokens(e);
                        e.indent_end();
                    } else {
                        arg.to_tokens(e);
                    }
                }

                if needs_indent && self.args.len() > 1 {
                    e.indent_end();
                }
            }
            BoolExprType::OrExpr => {
                // Add indentation for multi-line OR expressions in JOIN ON clauses
                let needs_indent = e.is_within_group(GroupKind::JoinExpr);
                if needs_indent && self.args.len() > 1 {
                    e.indent_start();
                }

                for (i, arg) in self.args.iter().enumerate() {
                    if i > 0 {
                        e.indent_start();
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::OR_KW);
                        e.space();
                        arg.to_tokens(e);
                        e.indent_end();
                    } else {
                        arg.to_tokens(e);
                    }
                }

                if needs_indent && self.args.len() > 1 {
                    e.indent_end();
                }
            }
            BoolExprType::NotExpr => {
                e.token(TokenKind::NOT_KW);
                e.space();
                if let Some(arg) = self.args.first() {
                    arg.to_tokens(e);
                }
            }
            BoolExprType::Undefined => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::CaseExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::CASE_KW);

        // arg is the test expression in CASE expr WHEN ...
        if let Some(ref arg) = self.arg {
            e.space();
            arg.to_tokens(e);
        }

        // args contains CaseWhen nodes
        for when_clause in &self.args {
            e.space();
            when_clause.to_tokens(e);
        }

        // defresult is the ELSE clause
        if let Some(ref defresult) = self.defresult {
            e.space();
            e.token(TokenKind::ELSE_KW);
            e.space();
            defresult.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::END_KW);
    }
}

impl ToTokens for pgt_query::protobuf::CaseWhen {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::WHEN_KW);
        e.space();

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::THEN_KW);
        e.space();

        if let Some(ref result) = self.result {
            result.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ArrayExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::ARRAY_KW);
        e.token(TokenKind::L_BRACK);

        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            element.to_tokens(e);
        }

        e.token(TokenKind::R_BRACK);
    }
}

impl ToTokens for pgt_query::protobuf::AArrayExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::ARRAY_KW);
        e.token(TokenKind::L_BRACK);

        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            element.to_tokens(e);
        }

        e.token(TokenKind::R_BRACK);
    }
}

impl ToTokens for pgt_query::protobuf::RowExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RowExpr);

        e.token(TokenKind::ROW_KW);
        e.token(TokenKind::L_PAREN);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SubLink {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::SubLinkType;

        match self.sub_link_type() {
            SubLinkType::ExistsSublink => {
                e.token(TokenKind::EXISTS_KW);
                e.space();
            }
            SubLinkType::AnySublink => {
                // testexpr is handled by the parent expression
                // we just need to output the subquery
            }
            SubLinkType::AllSublink => {
                // testexpr is handled by the parent expression
                // we just need to output the subquery
            }
            _ => {
                // For other types, just output the subselect
            }
        }

        e.token(TokenKind::L_PAREN);
        if let Some(ref subselect) = self.subselect {
            subselect.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::CoalesceExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::COALESCE_KW);
        e.token(TokenKind::L_PAREN);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::MinMaxExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::MinMaxOp;

        e.group_start(GroupKind::MinMaxExpr);

        match self.op() {
            MinMaxOp::IsGreatest => e.token(TokenKind::GREATEST_KW),
            MinMaxOp::IsLeast => e.token(TokenKind::LEAST_KW),
            MinMaxOp::Undefined => todo!(),
        }

        e.token(TokenKind::L_PAREN);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::XmlExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::XmlExprOp;

        e.group_start(GroupKind::XmlExpr);

        match self.op() {
            XmlExprOp::IsXmlelement => {
                e.token(TokenKind::XMLELEMENT_KW);
                e.token(TokenKind::L_PAREN);

                if !self.name.is_empty() {
                    e.token(TokenKind::NAME_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.name.clone()));

                    if !self.args.is_empty() {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                }

                for (i, arg) in self.args.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    arg.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsXmlconcat => {
                e.token(TokenKind::XMLCONCAT_KW);
                e.token(TokenKind::L_PAREN);

                for (i, arg) in self.args.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    arg.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsXmlforest => {
                e.token(TokenKind::XMLFOREST_KW);
                e.token(TokenKind::L_PAREN);

                for (i, (arg, name)) in self.args.iter().zip(self.arg_names.iter()).enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    arg.to_tokens(e);
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &name.node {
                        if !s.sval.is_empty() {
                            e.space();
                            e.token(TokenKind::AS_KW);
                            e.space();
                            e.token(TokenKind::IDENT(s.sval.clone()));
                        }
                    }
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsXmlparse => {
                e.token(TokenKind::XMLPARSE_KW);
                e.token(TokenKind::L_PAREN);
                e.token(TokenKind::DOCUMENT_KW);
                e.space();

                if let Some(first_arg) = self.args.first() {
                    first_arg.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsXmlpi => {
                e.token(TokenKind::XMLPI_KW);
                e.token(TokenKind::L_PAREN);
                e.token(TokenKind::NAME_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));

                if let Some(first_arg) = self.args.first() {
                    e.token(TokenKind::COMMA);
                    e.space();
                    first_arg.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsXmlroot => {
                e.token(TokenKind::XMLROOT_KW);
                e.token(TokenKind::L_PAREN);

                if let Some(xml_expr) = self.args.first() {
                    xml_expr.to_tokens(e);
                }

                if self.args.len() > 1 {
                    e.token(TokenKind::COMMA);
                    e.space();
                    e.token(TokenKind::VERSION_KW);
                    e.space();
                    if let Some(version) = self.args.get(1) {
                        version.to_tokens(e);
                    }
                }

                e.token(TokenKind::R_PAREN);
            }
            XmlExprOp::IsDocument => {
                if let Some(arg) = self.args.first() {
                    arg.to_tokens(e);
                }
                e.space();
                e.token(TokenKind::IS_KW);
                e.space();
                e.token(TokenKind::DOCUMENT_KW);
            }
            _ => todo!(),
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::NullTest {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::NullTestType;

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();

        match self.nulltesttype() {
            NullTestType::IsNull => {
                e.token(TokenKind::NULL_KW);
            }
            NullTestType::IsNotNull => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::NULL_KW);
            }
            NullTestType::Undefined => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::BooleanTest {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::BoolTestType;

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();

        match self.booltesttype() {
            BoolTestType::IsTrue => {
                e.token(TokenKind::TRUE_KW);
            }
            BoolTestType::IsNotTrue => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::TRUE_KW);
            }
            BoolTestType::IsFalse => {
                e.token(TokenKind::FALSE_KW);
            }
            BoolTestType::IsNotFalse => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::FALSE_KW);
            }
            BoolTestType::IsUnknown => {
                e.token(TokenKind::UNKNOWN_KW);
            }
            BoolTestType::IsNotUnknown => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::UNKNOWN_KW);
            }
            BoolTestType::Undefined => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::CreateConversionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateConversionStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.def {
            e.token(TokenKind::DEFAULT_KW);
            e.space();
        }

        e.token(TokenKind::CONVERSION_KW);
        e.space();

        for (i, name) in self.conversion_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.for_encoding_name)));
        e.space();
        e.token(TokenKind::TO_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.to_encoding_name)));
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();

        for (i, func) in self.func_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            func.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateCastStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateCastStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::CAST_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        if let Some(ref sourcetype) = self.sourcetype {
            sourcetype.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();

        if let Some(ref targettype) = self.targettype {
            targettype.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.space();

        if self.inout {
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::INOUT_KW);
        } else if let Some(ref func) = self.func {
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::FUNCTION_KW);
            e.space();
            func.to_tokens(e);
        } else {
            e.token(TokenKind::WITHOUT_KW);
            e.space();
            e.token(TokenKind::FUNCTION_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateTransformStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateTransformStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.replace {
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
            e.space();
        }

        e.token(TokenKind::TRANSFORM_KW);
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref type_name) = self.type_name {
            type_name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::LANGUAGE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.lang.clone()));

        e.space();
        e.token(TokenKind::L_PAREN);

        let mut needs_comma = false;

        if let Some(ref fromsql) = self.fromsql {
            e.token(TokenKind::FROM_KW);
            e.space();
            e.token(TokenKind::SQL_KW);
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::FUNCTION_KW);
            e.space();
            fromsql.to_tokens(e);
            needs_comma = true;
        }

        if let Some(ref tosql) = self.tosql {
            if needs_comma {
                e.token(TokenKind::COMMA);
                e.space();
            }
            e.token(TokenKind::TO_KW);
            e.space();
            e.token(TokenKind::SQL_KW);
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::FUNCTION_KW);
            e.space();
            tosql.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropOwnedStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropOwnedStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::OWNED_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();

        for (i, role) in self.roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            role.to_tokens(e);
        }

        use pgt_query::protobuf::DropBehavior;
        match self.behavior() {
            DropBehavior::DropRestrict => {
                e.space();
                e.token(TokenKind::RESTRICT_KW);
            }
            DropBehavior::DropCascade => {
                e.space();
                e.token(TokenKind::CASCADE_KW);
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ReassignOwnedStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ReassignOwnedStmt);

        e.token(TokenKind::REASSIGN_KW);
        e.space();
        e.token(TokenKind::OWNED_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();

        for (i, role) in self.roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            role.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::TO_KW);
        e.space();

        if let Some(ref newrole) = self.newrole {
            newrole.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTsDictionaryStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTsdictionaryStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TEXT_KW);
        e.space();
        e.token(TokenKind::SEARCH_KW);
        e.space();
        e.token(TokenKind::DICTIONARY_KW);
        e.space();

        for (i, name) in self.dictname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::L_PAREN);

        for (i, option) in self.options.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            option.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTsConfigurationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTsconfigurationStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TEXT_KW);
        e.space();
        e.token(TokenKind::SEARCH_KW);
        e.space();
        e.token(TokenKind::CONFIGURATION_KW);
        e.space();

        for (i, name) in self.cfgname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();

        use pgt_query::protobuf::AlterTsConfigType;
        match self.kind() {
            AlterTsConfigType::AlterTsconfigAddMapping => {
                e.token(TokenKind::ADD_KW);
                e.space();
                e.token(TokenKind::MAPPING_KW);
                e.space();
                e.token(TokenKind::FOR_KW);
                e.space();

                for (i, token) in self.tokentype.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    token.to_tokens(e);
                }

                e.space();
                e.token(TokenKind::WITH_KW);
                e.space();

                for (i, dict) in self.dicts.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    dict.to_tokens(e);
                }
            }
            AlterTsConfigType::AlterTsconfigAlterMappingForToken => {
                e.token(TokenKind::ALTER_KW);
                e.space();
                e.token(TokenKind::MAPPING_KW);
                e.space();
                e.token(TokenKind::FOR_KW);
                e.space();

                for (i, token) in self.tokentype.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    token.to_tokens(e);
                }

                if self.replace {
                    e.space();
                    e.token(TokenKind::REPLACE_KW);
                    e.space();
                }

                e.space();
                e.token(TokenKind::WITH_KW);
                e.space();

                for (i, dict) in self.dicts.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    dict.to_tokens(e);
                }
            }
            AlterTsConfigType::AlterTsconfigDropMapping => {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::MAPPING_KW);

                if self.missing_ok {
                    e.space();
                    e.token(TokenKind::IF_KW);
                    e.space();
                    e.token(TokenKind::EXISTS_KW);
                }

                e.space();
                e.token(TokenKind::FOR_KW);
                e.space();

                for (i, token) in self.tokentype.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    token.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreatePublicationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreatePublicationStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::PUBLICATION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.pubname.clone()));

        if self.for_all_tables {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
            e.space();
            e.token(TokenKind::TABLES_KW);
        } else if !self.pubobjects.is_empty() {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();

            todo!()
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);

            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterPublicationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterPublicationStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::PUBLICATION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.pubname.clone()));
        e.space();

        use pgt_query::protobuf::AlterPublicationAction;
        match self.action() {
            AlterPublicationAction::ApAddObjects => {
                e.token(TokenKind::ADD_KW);
                e.space();

                if self.for_all_tables {
                    e.token(TokenKind::ALL_KW);
                    e.space();
                    e.token(TokenKind::TABLES_KW);
                    e.space();
                    e.token(TokenKind::IN_KW);
                    e.space();
                    e.token(TokenKind::SCHEMA_KW);
                } else {
                    todo!()
                }
            }
            AlterPublicationAction::ApDropObjects => {
                e.token(TokenKind::DROP_KW);
                e.space();

                if self.for_all_tables {
                    e.token(TokenKind::ALL_KW);
                    e.space();
                    e.token(TokenKind::TABLES_KW);
                    e.space();
                    e.token(TokenKind::IN_KW);
                    e.space();
                    e.token(TokenKind::SCHEMA_KW);
                } else {
                    todo!()
                }
            }
            AlterPublicationAction::ApSetObjects => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.for_all_tables {
                    e.token(TokenKind::ALL_KW);
                    e.space();
                    e.token(TokenKind::TABLES_KW);
                    e.space();
                    e.token(TokenKind::IN_KW);
                    e.space();
                    e.token(TokenKind::SCHEMA_KW);
                } else {
                    e.token(TokenKind::TABLE_KW);
                    e.space();

                    for (i, obj) in self.pubobjects.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        obj.to_tokens(e);
                    }
                }
            }
            _ => {}
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);

            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PublicationObjSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PublicationObjSpec);

        use pgt_query::protobuf::PublicationObjSpecType;
        match self.pubobjtype() {
            PublicationObjSpecType::PublicationobjTable => {
                if let Some(ref pubtable) = self.pubtable {
                    if let Some(ref relation) = pubtable.relation {
                        relation.to_tokens(e);
                    }
                }
            }
            _ => todo!(),
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateSubscriptionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateSubscriptionStmt);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SUBSCRIPTION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.subname.clone()));
        e.space();
        e.token(TokenKind::CONNECTION_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.conninfo)));
        e.space();
        e.token(TokenKind::PUBLICATION_KW);
        e.space();

        for (i, pub_name) in self.publication.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            pub_name.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);

            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterSubscriptionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::AlterSubscriptionType;

        e.group_start(GroupKind::AlterSubscriptionStmt);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SUBSCRIPTION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.subname.clone()));
        e.space();

        match self.kind {
            x if x == AlterSubscriptionType::AlterSubscriptionOptions as i32 => {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::L_PAREN);

                for (i, option) in self.options.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    option.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            x if x == AlterSubscriptionType::AlterSubscriptionConnection as i32 => {
                e.token(TokenKind::CONNECTION_KW);
                e.space();
                e.token(TokenKind::STRING(format!("'{}'", self.conninfo)));
            }
            x if x == AlterSubscriptionType::AlterSubscriptionSetPublication as i32 => {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::PUBLICATION_KW);
                e.space();

                for (i, pub_name) in self.publication.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    pub_name.to_tokens(e);
                }

                if !self.options.is_empty() {
                    e.space();
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);

                    for (i, option) in self.options.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        option.to_tokens(e);
                    }

                    e.token(TokenKind::R_PAREN);
                }
            }
            x if x == AlterSubscriptionType::AlterSubscriptionAddPublication as i32 => {
                e.token(TokenKind::ADD_KW);
                e.space();
                e.token(TokenKind::PUBLICATION_KW);
                e.space();

                for (i, pub_name) in self.publication.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    pub_name.to_tokens(e);
                }

                if !self.options.is_empty() {
                    e.space();
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);

                    for (i, option) in self.options.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        option.to_tokens(e);
                    }

                    e.token(TokenKind::R_PAREN);
                }
            }
            x if x == AlterSubscriptionType::AlterSubscriptionDropPublication as i32 => {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::PUBLICATION_KW);
                e.space();

                for (i, pub_name) in self.publication.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    pub_name.to_tokens(e);
                }

                if !self.options.is_empty() {
                    e.space();
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);

                    for (i, option) in self.options.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        option.to_tokens(e);
                    }

                    e.token(TokenKind::R_PAREN);
                }
            }
            x if x == AlterSubscriptionType::AlterSubscriptionRefresh as i32 => {
                e.token(TokenKind::REFRESH_KW);
                e.space();
                e.token(TokenKind::PUBLICATION_KW);

                if !self.options.is_empty() {
                    e.space();
                    e.token(TokenKind::WITH_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);

                    for (i, option) in self.options.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        option.to_tokens(e);
                    }

                    e.token(TokenKind::R_PAREN);
                }
            }
            x if x == AlterSubscriptionType::AlterSubscriptionEnabled as i32 => {
                match self.options.first() {
                    Some(option) => {
                        option.to_tokens(e);
                    }
                    None => {}
                }
            }
            x if x == AlterSubscriptionType::AlterSubscriptionSkip as i32 => {
                e.token(TokenKind::SKIP_KW);
                e.space();
                e.token(TokenKind::L_PAREN);

                for (i, option) in self.options.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    option.to_tokens(e);
                }

                e.token(TokenKind::R_PAREN);
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::NamedArgExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::NamedArgExpr);

        e.token(TokenKind::IDENT(self.name.clone()));
        e.space();
        e.token(TokenKind::IDENT("=>".to_string()));
        e.space();

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropSubscriptionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropSubscriptionStmt);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::SUBSCRIPTION_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.subname.clone()));

        if self.behavior == pgt_query::protobuf::DropBehavior::DropCascade as i32 {
            e.space();
            e.token(TokenKind::CASCADE_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::WithClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::WithClause);

        e.token(TokenKind::WITH_KW);
        if self.recursive {
            e.space();
            e.token(TokenKind::RECURSIVE_KW);
        }
        e.space();

        for (i, cte) in self.ctes.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            cte.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CommonTableExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CommonTableExpr);

        e.token(TokenKind::IDENT(self.ctename.clone()));

        if !self.aliascolnames.is_empty() {
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.aliascolnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        if let Some(ref query) = self.ctequery {
            query.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::GroupingSet {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::GroupingSetKind;

        e.group_start(GroupKind::GroupingSet);

        match self.kind() {
            GroupingSetKind::GroupingSetEmpty => {
                e.token(TokenKind::L_PAREN);
                e.token(TokenKind::R_PAREN);
            }
            GroupingSetKind::GroupingSetSimple => {
                for (i, item) in self.content.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    item.to_tokens(e);
                }
            }
            GroupingSetKind::GroupingSetSets => {
                e.token(TokenKind::GROUPING_KW);
                e.space();
                e.token(TokenKind::SETS_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, item) in self.content.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    if let Some(pgt_query::protobuf::node::Node::GroupingSet(inner)) = &item.node {
                        if matches!(inner.kind(), GroupingSetKind::GroupingSetEmpty) {
                            e.token(TokenKind::L_PAREN);
                            e.token(TokenKind::R_PAREN);
                        } else {
                            item.to_tokens(e);
                        }
                    } else {
                        e.token(TokenKind::L_PAREN);
                        item.to_tokens(e);
                        e.token(TokenKind::R_PAREN);
                    }
                }
                e.token(TokenKind::R_PAREN);
            }
            GroupingSetKind::GroupingSetRollup => {
                e.token(TokenKind::ROLLUP_KW);
                e.token(TokenKind::L_PAREN);
                for (i, item) in self.content.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    item.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            GroupingSetKind::GroupingSetCube => {
                e.token(TokenKind::CUBE_KW);
                e.token(TokenKind::L_PAREN);
                for (i, item) in self.content.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    item.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            GroupingSetKind::Undefined => {
                for (i, item) in self.content.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    item.to_tokens(e);
                }
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AIndirection {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AIndirection);

        if let Some(ref arg) = self.arg {
            // Check if we need parentheses around the arg
            let needs_parens =
                matches!(&arg.node, Some(pgt_query::protobuf::node::Node::RowExpr(_)));

            if needs_parens {
                e.token(TokenKind::L_PAREN);
            }
            arg.to_tokens(e);
            if needs_parens {
                e.token(TokenKind::R_PAREN);
            }
        }

        for ind in &self.indirection {
            match &ind.node {
                Some(pgt_query::protobuf::node::Node::String(s)) => {
                    e.token(TokenKind::DOT);
                    e.token(TokenKind::IDENT(s.sval.clone()));
                }
                Some(pgt_query::protobuf::node::Node::AIndices(indices)) => {
                    e.token(TokenKind::L_BRACK);
                    if let Some(ref lidx) = indices.lidx {
                        lidx.to_tokens(e);
                    }
                    if indices.is_slice {
                        e.token(TokenKind::IDENT(":".to_string()));
                        if let Some(ref uidx) = indices.uidx {
                            uidx.to_tokens(e);
                        }
                    }
                    e.token(TokenKind::R_BRACK);
                }
                Some(pgt_query::protobuf::node::Node::AStar(_)) => {
                    e.token(TokenKind::DOT);
                    e.token(TokenKind::IDENT("*".to_string()));
                }
                _ => ind.to_tokens(e),
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AIndices {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::L_BRACK);

        if self.is_slice {
            if let Some(ref lidx) = self.lidx {
                lidx.to_tokens(e);
            }
            e.token(TokenKind::IDENT(":".to_string()));
            if let Some(ref uidx) = self.uidx {
                uidx.to_tokens(e);
            }
        } else {
            if let Some(ref uidx) = self.uidx {
                uidx.to_tokens(e);
            } else if let Some(ref lidx) = self.lidx {
                lidx.to_tokens(e);
            }
        }

        e.token(TokenKind::R_BRACK);
    }
}

impl ToTokens for pgt_query::protobuf::LockingClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::{LockClauseStrength, LockWaitPolicy};

        e.group_start(GroupKind::LockingClause);

        e.token(TokenKind::FOR_KW);
        e.space();

        match self.strength() {
            LockClauseStrength::LcsNone | LockClauseStrength::Undefined => {}
            LockClauseStrength::LcsForupdate => e.token(TokenKind::UPDATE_KW),
            LockClauseStrength::LcsFornokeyupdate => {
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
                e.space();
                e.token(TokenKind::UPDATE_KW);
            }
            LockClauseStrength::LcsForshare => e.token(TokenKind::SHARE_KW),
            LockClauseStrength::LcsForkeyshare => {
                e.token(TokenKind::KEY_KW);
                e.space();
                e.token(TokenKind::SHARE_KW);
            }
        }

        if !self.locked_rels.is_empty() {
            e.space();
            e.token(TokenKind::OF_KW);
            e.space();
            for (i, rel) in self.locked_rels.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                rel.to_tokens(e);
            }
        }

        match self.wait_policy() {
            LockWaitPolicy::LockWaitBlock | LockWaitPolicy::Undefined => {}
            LockWaitPolicy::LockWaitSkip => {
                e.space();
                e.token(TokenKind::SKIP_KW);
                e.space();
                e.token(TokenKind::LOCKED_KW);
            }
            LockWaitPolicy::LockWaitError => {
                e.space();
                e.token(TokenKind::NOWAIT_KW);
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::TableFunc {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TableFunc);

        match self.functype() {
            pgt_query::protobuf::TableFuncType::TftJsonTable => {
                e.token(TokenKind::IDENT("JSON_TABLE".to_string()));
            }
            pgt_query::protobuf::TableFuncType::TftXmltable => {
                e.token(TokenKind::IDENT("XMLTABLE".to_string()));
            }
            _ => todo!("Unknown table function type"),
        }

        e.token(TokenKind::L_PAREN);

        if let Some(ref docexpr) = self.docexpr {
            docexpr.to_tokens(e);
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        if let Some(ref rowexpr) = self.rowexpr {
            rowexpr.to_tokens(e);
            e.line(LineType::SoftOrSpace);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonTable {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonTable);

        e.token(TokenKind::IDENT("JSON_TABLE".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref context_item) = self.context_item {
            context_item.to_tokens(e);
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }

        if let Some(ref pathspec) = self.pathspec {
            pathspec.to_tokens(e);
        }

        if !self.columns.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::COLUMNS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.line(LineType::Hard);
            e.indent_start();

            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::Hard);
                }
                col.to_tokens(e);
            }

            e.indent_end();
            e.line(LineType::Hard);
            e.token(TokenKind::R_PAREN);
        }

        e.token(TokenKind::R_PAREN);

        if let Some(ref alias) = self.alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            alias.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonValueExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref raw_expr) = self.raw_expr {
            raw_expr.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonTablePathSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref string) = self.string {
            string.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonTableColumn {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.name.clone()));
        e.space();

        if let Some(ref type_name) = self.type_name {
            type_name.to_tokens(e);
        }

        if let Some(ref pathspec) = self.pathspec {
            e.space();
            e.token(TokenKind::PATH_KW);
            e.space();
            pathspec.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::DistinctExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DistinctExpr);

        if !self.args.is_empty() {
            if self.args.len() >= 2 {
                self.args[0].to_tokens(e);
                e.space();
                e.token(TokenKind::IS_KW);
                e.space();
                e.token(TokenKind::DISTINCT_KW);
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
                self.args[1].to_tokens(e);
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::NullIfExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::NullIfExpr);

        e.token(TokenKind::NULLIF_KW);
        e.token(TokenKind::L_PAREN);

        if self.args.len() >= 2 {
            self.args[0].to_tokens(e);
            e.token(TokenKind::COMMA);
            e.space();
            self.args[1].to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SqlValueFunction {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::SqlValueFunctionOp;

        match self.op() {
            SqlValueFunctionOp::SvfopCurrentDate => e.token(TokenKind::CURRENT_DATE_KW),
            SqlValueFunctionOp::SvfopCurrentTime => e.token(TokenKind::CURRENT_TIME_KW),
            SqlValueFunctionOp::SvfopCurrentTimeN => e.token(TokenKind::CURRENT_TIME_KW),
            SqlValueFunctionOp::SvfopCurrentTimestamp => e.token(TokenKind::CURRENT_TIMESTAMP_KW),
            SqlValueFunctionOp::SvfopCurrentTimestampN => e.token(TokenKind::CURRENT_TIMESTAMP_KW),
            SqlValueFunctionOp::SvfopLocaltime => e.token(TokenKind::LOCALTIME_KW),
            SqlValueFunctionOp::SvfopLocaltimeN => e.token(TokenKind::LOCALTIME_KW),
            SqlValueFunctionOp::SvfopLocaltimestamp => e.token(TokenKind::LOCALTIMESTAMP_KW),
            SqlValueFunctionOp::SvfopLocaltimestampN => e.token(TokenKind::LOCALTIMESTAMP_KW),
            SqlValueFunctionOp::SvfopCurrentRole => e.token(TokenKind::CURRENT_ROLE_KW),
            SqlValueFunctionOp::SvfopCurrentUser => e.token(TokenKind::CURRENT_USER_KW),
            SqlValueFunctionOp::SvfopUser => e.token(TokenKind::USER_KW),
            SqlValueFunctionOp::SvfopSessionUser => e.token(TokenKind::SESSION_USER_KW),
            SqlValueFunctionOp::SvfopCurrentCatalog => e.token(TokenKind::CURRENT_CATALOG_KW),
            SqlValueFunctionOp::SvfopCurrentSchema => e.token(TokenKind::CURRENT_SCHEMA_KW),
            SqlValueFunctionOp::SqlvalueFunctionOpUndefined => todo!(),
        }
    }
}

impl ToTokens for pgt_query::protobuf::CollateExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
        e.space();
        e.token(TokenKind::COLLATE_KW);
        e.space();
        e.token(TokenKind::STRING("en_US".to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::IntoClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IntoClause);

        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();

        if let Some(ref rel) = self.rel {
            rel.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PartitionElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PartitionElem);

        if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
        } else if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PartitionSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PartitionSpec);

        e.space();
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();

        use pgt_query::protobuf::PartitionStrategy;
        match self.strategy() {
            PartitionStrategy::List => e.token(TokenKind::IDENT("LIST".to_string())),
            PartitionStrategy::Range => e.token(TokenKind::RANGE_KW),
            PartitionStrategy::Hash => e.token(TokenKind::IDENT("HASH".to_string())),
            _ => {}
        }

        e.space();
        e.token(TokenKind::L_PAREN);

        for (i, param) in self.part_params.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            param.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::PartitionBoundSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::PartitionBoundSpec);

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::VALUES_KW);
        e.space();

        if self.is_default {
            e.token(TokenKind::DEFAULT_KW);
        } else if !self.listdatums.is_empty() {
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, datum) in self.listdatums.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                datum.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        } else if !self.lowerdatums.is_empty() && !self.upperdatums.is_empty() {
            e.token(TokenKind::FROM_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, datum) in self.lowerdatums.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                datum.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, datum) in self.upperdatums.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                datum.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        } else if self.modulus > 0 {
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT("MODULUS".to_string()));
            e.space();
            e.token(TokenKind::IDENT(self.modulus.to_string()));
            e.token(TokenKind::COMMA);
            e.space();
            e.token(TokenKind::IDENT("REMAINDER".to_string()));
            e.space();
            e.token(TokenKind::IDENT(self.remainder.to_string()));
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SetToDefault {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::DEFAULT_KW);
    }
}

impl ToTokens for pgt_query::protobuf::TableLikeClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::LIKE_KW);
        e.space();
        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        let options = self.options;
        if options == 2147483647 {
            e.space();
            e.token(TokenKind::INCLUDING_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
        } else {
            if options & 1 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("DEFAULTS".to_string()));
            }
            if options & 2 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("IDENTITY".to_string()));
            }
            if options & 4 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("INDEXES".to_string()));
            }
            if options & 8 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("STORAGE".to_string()));
            }
            if options & 16 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("COMMENTS".to_string()));
            }
            if options & 32 != 0 {
                e.space();
                e.token(TokenKind::INCLUDING_KW);
                e.space();
                e.token(TokenKind::IDENT("STATISTICS".to_string()));
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::InferClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::InferClause);

        e.token(TokenKind::L_PAREN);
        for (i, elem) in self.index_elems.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            elem.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.as_ref().to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::OnConflictClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::OnConflictAction;

        e.group_start(GroupKind::OnConflictClause);

        e.token(TokenKind::ON_KW);
        e.space();
        e.token(TokenKind::CONFLICT_KW);

        if let Some(ref infer) = self.infer {
            e.space();
            infer.to_tokens(e);
        }

        e.space();

        match self.action() {
            OnConflictAction::OnconflictNothing => {
                e.token(TokenKind::DO_KW);
                e.space();
                e.token(TokenKind::NOTHING_KW);
            }
            OnConflictAction::OnconflictUpdate => {
                e.token(TokenKind::DO_KW);
                e.space();
                e.token(TokenKind::UPDATE_KW);
                e.space();
                e.token(TokenKind::SET_KW);
                e.space();

                for (i, target) in self.target_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    target.to_tokens(e);
                }

                if let Some(ref where_clause) = self.where_clause {
                    e.space();
                    e.token(TokenKind::WHERE_KW);
                    e.space();
                    where_clause.as_ref().to_tokens(e);
                }
            }
            _ => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Var {
    fn to_tokens(&self, _e: &mut EventEmitter) {
        todo!()
    }
}

impl ToTokens for pgt_query::protobuf::NextValueExpr {
    fn to_tokens(&self, _e: &mut EventEmitter) {
        todo!()
    }
}

impl ToTokens for pgt_query::protobuf::InferenceElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::FromExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        for (i, from_item) in self.fromlist.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            from_item.to_tokens(e);
        }

        if let Some(ref quals) = self.quals {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            quals.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::RangeTblRef {
    fn to_tokens(&self, _e: &mut EventEmitter) {
        todo!()
    }
}

impl ToTokens for pgt_query::protobuf::TargetEntry {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        if !self.resname.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT(self.resname.clone()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::Query {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::Query);

        use pgt_query::protobuf::CmdType;

        match self.command_type() {
            CmdType::CmdSelect => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                } else {
                    e.token(TokenKind::SELECT_KW);
                    e.space();

                    for (i, target) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        target.to_tokens(e);
                    }
                }
            }
            CmdType::CmdInsert => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                }
            }
            CmdType::CmdUpdate => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                }
            }
            CmdType::CmdDelete => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                }
            }
            CmdType::CmdMerge => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                }
            }
            CmdType::CmdUtility => {
                if let Some(ref utility_stmt) = self.utility_stmt {
                    utility_stmt.to_tokens(e);
                }
            }
            _ => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::OnConflictExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::OnConflictExpr);

        use pgt_query::protobuf::OnConflictAction;

        match self.action() {
            OnConflictAction::OnconflictNothing => {
                e.token(TokenKind::NOTHING_KW);
            }
            OnConflictAction::OnconflictUpdate => {
                e.token(TokenKind::UPDATE_KW);
                e.space();
                e.token(TokenKind::SET_KW);
                e.space();

                for (i, set_item) in self.on_conflict_set.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    set_item.to_tokens(e);
                }

                if let Some(ref where_clause) = self.on_conflict_where {
                    e.space();
                    e.token(TokenKind::WHERE_KW);
                    e.space();
                    where_clause.to_tokens(e);
                }
            }
            _ => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CurrentOfExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::CURRENT_KW);
        e.space();
        e.token(TokenKind::OF_KW);
        e.space();
        if !self.cursor_name.is_empty() {
            e.token(TokenKind::IDENT(self.cursor_name.clone()));
        } else if self.cursor_param != 0 {
            e.token(TokenKind::IDENT(format!("${}", self.cursor_param)));
        }
    }
}

impl ToTokens for pgt_query::protobuf::SubscriptingRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.refexpr {
            expr.to_tokens(e);
        }

        e.token(TokenKind::L_BRACK);

        if !self.refupperindexpr.is_empty() {
            for (i, idx) in self.refupperindexpr.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                idx.to_tokens(e);
            }
        }

        if !self.reflowerindexpr.is_empty() {
            e.token(TokenKind::IDENT(":".to_string()));
            for (i, idx) in self.reflowerindexpr.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                idx.to_tokens(e);
            }
        }

        e.token(TokenKind::R_BRACK);
    }
}

impl ToTokens for pgt_query::protobuf::WindowFunc {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if self.winfnoid == 0 {
            return;
        }

        e.token(TokenKind::IDENT("row_number".to_string()));
        e.token(TokenKind::L_PAREN);

        if self.winstar {
            e.token(TokenKind::IDENT("*".to_string()));
        } else if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
        }

        e.token(TokenKind::R_PAREN);

        e.space();
        e.token(TokenKind::OVER_KW);
        e.space();
        e.token(TokenKind::L_PAREN);

        if self.winref > 0 {
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            e.token(TokenKind::IDENT("id".to_string()));
        }

        e.token(TokenKind::R_PAREN);

        if let Some(ref filter) = self.aggfilter {
            e.space();
            e.token(TokenKind::FILTER_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::WHERE_KW);
            e.space();
            filter.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::GroupingFunc {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("grouping".to_string()));
        e.token(TokenKind::L_PAREN);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::Aggref {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if self.aggfnoid == 0 {
            return;
        }

        e.token(TokenKind::IDENT("count".to_string()));
        e.token(TokenKind::L_PAREN);

        if self.aggstar {
            e.token(TokenKind::IDENT("*".to_string()));
        } else if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
        }

        e.token(TokenKind::R_PAREN);

        if let Some(ref filter) = self.aggfilter {
            e.space();
            e.token(TokenKind::FILTER_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::WHERE_KW);
            e.space();
            filter.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::FuncExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        // TODO: Implement proper function name resolution using funcid
        // For now, just emit the arguments
        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonArrayConstructor {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("JSON_ARRAY".to_string()));
        e.token(TokenKind::L_PAREN);

        for (i, expr) in self.exprs.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            expr.to_tokens(e);
        }

        if self.absent_on_null {
            if !self.exprs.is_empty() {
                e.space();
            }
            e.token(TokenKind::IDENT("ABSENT".to_string()));
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }

        if let Some(ref output) = self.output {
            e.space();
            output.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::JsonObjectConstructor {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("JSON_OBJECT".to_string()));
        e.token(TokenKind::L_PAREN);

        for (i, expr) in self.exprs.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            expr.to_tokens(e);
        }

        if self.absent_on_null {
            if !self.exprs.is_empty() {
                e.space();
            }
            e.token(TokenKind::IDENT("ABSENT".to_string()));
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }

        if self.unique {
            if !self.exprs.is_empty() || self.absent_on_null {
                e.space();
            }
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNIQUE_KW);
            e.space();
            e.token(TokenKind::IDENT("KEYS".to_string()));
        }

        if let Some(ref output) = self.output {
            e.space();
            output.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::JsonKeyValue {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref key) = self.key {
            key.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IDENT(":".to_string()));
        e.space();

        if let Some(ref value) = self.value {
            value.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonOutput {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref returning) = self.returning {
            returning.to_tokens(e);
        }

        if let Some(ref type_name) = self.type_name {
            e.space();
            type_name.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonFuncExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::JsonExprOp;

        match self.op() {
            JsonExprOp::JsonExistsOp => {
                e.token(TokenKind::IDENT("JSON_EXISTS".to_string()));
            }
            JsonExprOp::JsonQueryOp => {
                e.token(TokenKind::IDENT("JSON_QUERY".to_string()));
            }
            JsonExprOp::JsonValueOp => {
                e.token(TokenKind::IDENT("JSON_VALUE".to_string()));
            }
            _ => {}
        }

        e.token(TokenKind::L_PAREN);

        if let Some(ref context_item) = self.context_item {
            context_item.to_tokens(e);
        }

        if let Some(ref pathspec) = self.pathspec {
            e.token(TokenKind::COMMA);
            e.space();
            pathspec.to_tokens(e);
        }

        if !self.passing.is_empty() {
            e.space();
            e.token(TokenKind::IDENT("PASSING".to_string()));
            e.space();

            for (i, pass) in self.passing.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                pass.to_tokens(e);
            }
        }

        if let Some(ref output) = self.output {
            e.space();
            output.to_tokens(e);
        }

        if let Some(ref on_empty) = self.on_empty {
            e.space();
            on_empty.to_tokens(e);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::IDENT("EMPTY".to_string()));
        }

        if let Some(ref on_error) = self.on_error {
            e.space();
            on_error.to_tokens(e);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::ERROR_KW);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::JsonExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::JsonExprOp;

        match self.op() {
            JsonExprOp::JsonExistsOp => {
                e.token(TokenKind::IDENT("JSON_EXISTS".to_string()));
            }
            JsonExprOp::JsonQueryOp => {
                e.token(TokenKind::IDENT("JSON_QUERY".to_string()));
            }
            JsonExprOp::JsonValueOp => {
                e.token(TokenKind::IDENT("JSON_VALUE".to_string()));
            }
            _ => {}
        }

        e.token(TokenKind::L_PAREN);

        if let Some(ref formatted_expr) = self.formatted_expr {
            formatted_expr.to_tokens(e);
        }

        if let Some(ref path_spec) = self.path_spec {
            e.token(TokenKind::COMMA);
            e.space();
            path_spec.to_tokens(e);
        }

        if !self.passing_names.is_empty() && !self.passing_values.is_empty() {
            e.space();
            e.token(TokenKind::IDENT("PASSING".to_string()));
            e.space();

            for (i, (name, value)) in self
                .passing_names
                .iter()
                .zip(self.passing_values.iter())
                .enumerate()
            {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                value.to_tokens(e);
                e.space();
                e.token(TokenKind::AS_KW);
                e.space();
                name.to_tokens(e);
            }
        }

        if let Some(ref returning) = self.returning {
            e.space();
            returning.to_tokens(e);
        }

        if let Some(ref on_empty) = self.on_empty {
            e.space();
            on_empty.to_tokens(e);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::IDENT("EMPTY".to_string()));
        }

        if let Some(ref on_error) = self.on_error {
            e.space();
            on_error.to_tokens(e);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::ERROR_KW);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::RangeTableFunc {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::RangeTableFunc);

        if self.lateral {
            e.token(TokenKind::LATERAL_KW);
            e.space();
        }

        e.token(TokenKind::IDENT("xmltable".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref rowexpr) = self.rowexpr {
            rowexpr.to_tokens(e);
        }

        if let Some(ref docexpr) = self.docexpr {
            e.space();
            e.token(TokenKind::IDENT("passing".to_string()));
            e.space();
            docexpr.to_tokens(e);
        }

        if !self.columns.is_empty() {
            e.space();
            e.token(TokenKind::IDENT("columns".to_string()));
            e.space();
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
        }

        e.token(TokenKind::R_PAREN);

        if let Some(ref alias) = self.alias {
            e.space();
            alias.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonBehavior {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::JsonBehaviorType;

        match self.btype() {
            JsonBehaviorType::JsonBehaviorNull => {
                e.token(TokenKind::NULL_KW);
            }
            JsonBehaviorType::JsonBehaviorError => {
                e.token(TokenKind::ERROR_KW);
            }
            JsonBehaviorType::JsonBehaviorEmpty => {
                e.token(TokenKind::IDENT("EMPTY".to_string()));
            }
            JsonBehaviorType::JsonBehaviorEmptyArray => {
                e.token(TokenKind::IDENT("EMPTY".to_string()));
                e.space();
                e.token(TokenKind::ARRAY_KW);
            }
            JsonBehaviorType::JsonBehaviorEmptyObject => {
                e.token(TokenKind::IDENT("EMPTY".to_string()));
                e.space();
                e.token(TokenKind::IDENT("OBJECT".to_string()));
            }
            JsonBehaviorType::JsonBehaviorDefault => {
                e.token(TokenKind::DEFAULT_KW);
                if let Some(ref expr) = self.expr {
                    e.space();
                    expr.to_tokens(e);
                }
            }
            _ => {}
        }

        if self.coerce {
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::IDENT("EMPTY".to_string()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonReturning {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::RETURNING_KW);
        e.space();

        if let Some(ref format) = self.format {
            format.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonIsPredicate {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();

        use pgt_query::protobuf::JsonValueType;
        match self.item_type() {
            JsonValueType::JsTypeObject => {
                e.token(TokenKind::JSON_KW);
                e.space();
                e.token(TokenKind::IDENT("OBJECT".to_string()));
            }
            JsonValueType::JsTypeArray => {
                e.token(TokenKind::JSON_KW);
                e.space();
                e.token(TokenKind::ARRAY_KW);
            }
            JsonValueType::JsTypeScalar => {
                e.token(TokenKind::JSON_KW);
                e.space();
                e.token(TokenKind::IDENT("SCALAR".to_string()));
            }
            _ => {
                e.token(TokenKind::JSON_KW);
            }
        }

        if self.unique_keys {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNIQUE_KW);
            e.space();
            e.token(TokenKind::IDENT("KEYS".to_string()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonFormat {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::{JsonEncoding, JsonFormatType};

        e.token(TokenKind::FORMAT_KW);
        e.space();

        match self.format_type() {
            JsonFormatType::JsFormatJson => {
                e.token(TokenKind::JSON_KW);
            }
            JsonFormatType::JsFormatJsonb => {
                e.token(TokenKind::IDENT("JSONB".to_string()));
            }
            _ => {}
        }

        match self.encoding() {
            JsonEncoding::JsEncUtf8 => {
                e.space();
                e.token(TokenKind::IDENT("ENCODING".to_string()));
                e.space();
                e.token(TokenKind::IDENT("UTF8".to_string()));
            }
            JsonEncoding::JsEncUtf16 => {
                e.space();
                e.token(TokenKind::IDENT("ENCODING".to_string()));
                e.space();
                e.token(TokenKind::IDENT("UTF16".to_string()));
            }
            JsonEncoding::JsEncUtf32 => {
                e.space();
                e.token(TokenKind::IDENT("ENCODING".to_string()));
                e.space();
                e.token(TokenKind::IDENT("UTF32".to_string()));
            }
            _ => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::XmlSerialize {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("XMLSERIALIZE".to_string()));
        e.token(TokenKind::L_PAREN);

        use pgt_query::protobuf::XmlOptionType;
        match self.xmloption() {
            XmlOptionType::XmloptionDocument => {
                e.token(TokenKind::IDENT("DOCUMENT".to_string()));
            }
            XmlOptionType::XmloptionContent => {
                e.token(TokenKind::IDENT("CONTENT".to_string()));
            }
            _ => {}
        }

        if let Some(ref expr) = self.expr {
            e.space();
            expr.to_tokens(e);
        }

        if let Some(ref type_name) = self.type_name {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            type_name.to_tokens(e);
        }

        if self.indent {
            e.space();
            e.token(TokenKind::IDENT("INDENT".to_string()));
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::RangeTableFuncCol {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.colname.clone()));

        if let Some(ref type_name) = self.type_name {
            e.space();
            type_name.to_tokens(e);
        }

        if self.for_ordinality {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::IDENT("ordinality".to_string()));
        }

        if let Some(ref colexpr) = self.colexpr {
            e.space();
            e.token(TokenKind::IDENT("path".to_string()));
            e.space();
            colexpr.to_tokens(e);
        }

        if let Some(ref coldefexpr) = self.coldefexpr {
            e.space();
            e.token(TokenKind::DEFAULT_KW);
            e.space();
            coldefexpr.to_tokens(e);
        }

        if self.is_not_null {
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }
    }
}

impl ToTokens for pgt_query::protobuf::RangeTableSample {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IDENT("TABLESAMPLE".to_string()));
        e.space();

        for (i, method) in self.method.iter().enumerate() {
            if i > 0 {
                e.space();
            }
            method.to_tokens(e);
        }

        if !self.args.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref repeatable) = self.repeatable {
            e.space();
            e.token(TokenKind::IDENT("REPEATABLE".to_string()));
            e.space();
            e.token(TokenKind::L_PAREN);
            repeatable.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }
    }
}

impl ToTokens for pgt_query::protobuf::RelabelType {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::CoerceToDomain {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::FieldSelect {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::PartitionRangeDatum {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::PartitionRangeDatumKind;

        match self.kind() {
            PartitionRangeDatumKind::PartitionRangeDatumMinvalue => {
                e.token(TokenKind::IDENT("MINVALUE".to_string()));
            }
            PartitionRangeDatumKind::PartitionRangeDatumMaxvalue => {
                e.token(TokenKind::IDENT("MAXVALUE".to_string()));
            }
            PartitionRangeDatumKind::PartitionRangeDatumValue => {
                if let Some(ref value) = self.value {
                    value.to_tokens(e);
                }
            }
            _ => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::CteSearchClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::SEARCH_KW);
        e.space();

        if self.search_breadth_first {
            e.token(TokenKind::IDENT("BREADTH".to_string()));
        } else {
            e.token(TokenKind::IDENT("DEPTH".to_string()));
        }
        e.space();
        e.token(TokenKind::FIRST_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();

        for (i, col) in self.search_col_list.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            col.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::IDENT(self.search_seq_column.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::CteCycleClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::CYCLE_KW);
        e.space();

        for (i, col) in self.cycle_col_list.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            col.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::IDENT(self.cycle_mark_column.clone()));

        if let Some(ref value) = self.cycle_mark_value {
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            value.to_tokens(e);
        }

        if let Some(ref default) = self.cycle_mark_default {
            e.space();
            e.token(TokenKind::DEFAULT_KW);
            e.space();
            default.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.cycle_path_column.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::TriggerTransition {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if self.is_new {
            e.token(TokenKind::NEW_KW);
        } else {
            e.token(TokenKind::OLD_KW);
        }

        if self.is_table {
            e.space();
            e.token(TokenKind::TABLE_KW);
        }

        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::IDENT(self.name.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::JsonArgument {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref val) = self.val {
            val.to_tokens(e);
        }

        if !self.name.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT(self.name.clone()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::PublicationTable {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.columns.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::CoerceViaIo {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::FieldStore {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ArrayCoerceExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::ConvertRowtypeExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::CaseTestExpr {
    fn to_tokens(&self, _e: &mut EventEmitter) {
        todo!()
    }
}

impl ToTokens for pgt_query::protobuf::CoerceToDomainValue {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::VALUE_KW);
    }
}

impl ToTokens for pgt_query::protobuf::MergeAction {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::CmdType;

        match self.command_type() {
            CmdType::CmdInsert => {
                e.token(TokenKind::INSERT_KW);

                if !self.target_list.is_empty() {
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, col) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        col.to_tokens(e);
                    }
                    e.token(TokenKind::R_PAREN);
                    e.space();
                    e.token(TokenKind::VALUES_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, _) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        e.token(TokenKind::DEFAULT_KW);
                    }
                    e.token(TokenKind::R_PAREN);
                } else {
                    e.space();
                    e.token(TokenKind::DEFAULT_KW);
                    e.space();
                    e.token(TokenKind::VALUES_KW);
                }
            }
            CmdType::CmdUpdate => {
                e.token(TokenKind::UPDATE_KW);
                e.space();
                e.token(TokenKind::SET_KW);
                e.space();

                for (i, col) in self.target_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    col.to_tokens(e);
                }
            }
            CmdType::CmdDelete => {
                e.token(TokenKind::DELETE_KW);
            }
            CmdType::CmdNothing => {
                e.token(TokenKind::IDENT("DO".to_string()));
                e.space();
                e.token(TokenKind::NOTHING_KW);
            }
            _ => {}
        }
    }
}

impl ToTokens for pgt_query::protobuf::PartitionCmd {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref name) = self.name {
            name.to_tokens(e);
        }

        if let Some(ref bound) = self.bound {
            e.space();
            bound.to_tokens(e);
        }

        if self.concurrent {
            e.space();
            e.token(TokenKind::IDENT("CONCURRENTLY".to_string()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::JsonConstructorExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::JsonConstructorType;

        match self.r#type() {
            JsonConstructorType::JsctorJsonObject => {
                e.token(TokenKind::IDENT("JSON_OBJECT".to_string()));
            }
            JsonConstructorType::JsctorJsonArray => {
                e.token(TokenKind::IDENT("JSON_ARRAY".to_string()));
            }
            _ => {}
        }

        e.token(TokenKind::L_PAREN);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            arg.to_tokens(e);
        }

        if self.absent_on_null {
            if !self.args.is_empty() {
                e.space();
            }
            e.token(TokenKind::IDENT("ABSENT".to_string()));
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }

        if self.unique {
            if !self.args.is_empty() || self.absent_on_null {
                e.space();
            }
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNIQUE_KW);
            e.space();
            e.token(TokenKind::IDENT("KEYS".to_string()));
        }

        if let Some(ref returning) = self.returning {
            e.space();
            e.token(TokenKind::IDENT("RETURNING".to_string()));
            e.space();
            returning.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

impl ToTokens for pgt_query::protobuf::JsonParseExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("JSON_PARSE".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        if self.unique_keys {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNIQUE_KW);
            e.space();
            e.token(TokenKind::IDENT("KEYS".to_string()));
        }

        if let Some(ref output) = self.output {
            e.space();
            e.token(TokenKind::IDENT("RETURNING".to_string()));
            e.space();
            output.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);
    }
}

#[cfg(test)]
mod test {
    use crate::emitter::{EventEmitter, ToTokens};

    use insta::assert_debug_snapshot;

    #[test]
    fn simple_select() {
        let input = "select public.t.a as y, b as z, c from t where id = @id;";

        let parsed = pgt_query::parse(input).expect("Failed to parse SQL");

        let ast = parsed.root().expect("No root node found");

        let mut emitter = EventEmitter::new();
        ast.to_tokens(&mut emitter);

        assert_debug_snapshot!(emitter.events);
    }
}

impl ToTokens for pgt_query::protobuf::SinglePartitionSpec {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::AlternativeSubPlan {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlternativeSubPlan);

        if let Some(ref first_plan) = self.subplans.first() {
            first_plan.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CallContext {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::MergeSupportFunc {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::SubPlan {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SubPlan);

        if let Some(ref _param_id) = self.param_ids.first() {
            e.token(TokenKind::IDENT(format!("$SUBPLAN{}", self.plan_id)));
        } else if !self.plan_name.is_empty() {
            e.token(TokenKind::IDENT(self.plan_name.clone()));
        } else {
            e.token(TokenKind::IDENT(format!("SubPlan {}", self.plan_id)));
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonTablePath {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.name.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::JsonTablePathScan {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonTablePathScan);

        if let Some(ref plan) = self.plan {
            plan.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonTableSiblingJoin {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonTableSiblingJoin);

        if let Some(ref lplan) = self.lplan {
            lplan.to_tokens(e);
        }

        if let Some(ref rplan) = self.rplan {
            e.space();
            e.token(TokenKind::COMMA);
            e.space();
            rplan.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonScalarExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonScalarExpr);

        e.token(TokenKind::IDENT("JSON_SCALAR".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        if let Some(ref output) = self.output {
            e.space();
            output.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonSerializeExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonSerializeExpr);

        e.token(TokenKind::IDENT("JSON_SERIALIZE".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        }

        if let Some(ref output) = self.output {
            e.space();
            output.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonArrayQueryConstructor {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonArrayQueryConstructor);

        e.token(TokenKind::IDENT("JSON_ARRAY".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref query) = self.query {
            query.to_tokens(e);
        }

        if let Some(ref format) = self.format {
            e.space();
            format.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonAggConstructor {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonAggConstructor);

        if let Some(ref output) = self.output {
            output.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonObjectAgg {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonObjectAgg);

        e.token(TokenKind::IDENT("JSON_OBJECTAGG".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        if self.absent_on_null {
            e.space();
            e.token(TokenKind::ABSENT_KW);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }

        if self.unique {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::UNIQUE_KW);
            e.space();
            e.token(TokenKind::KEYS_KW);
        }

        if let Some(ref constructor) = self.constructor {
            e.space();
            constructor.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::JsonArrayAgg {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::JsonArrayAgg);

        e.token(TokenKind::IDENT("JSON_ARRAYAGG".to_string()));
        e.token(TokenKind::L_PAREN);

        if let Some(ref arg) = self.arg {
            arg.to_tokens(e);
        }

        if self.absent_on_null {
            e.space();
            e.token(TokenKind::ABSENT_KW);
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }

        if let Some(ref constructor) = self.constructor {
            e.space();
            constructor.to_tokens(e);
        }

        e.token(TokenKind::R_PAREN);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::WindowClause {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::WindowFuncRunCondition {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::SortGroupClause {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::RowMarkClause {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::WithCheckOption {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::TableSampleClause {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::RangeTblEntry {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::RtePermissionInfo {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}

impl ToTokens for pgt_query::protobuf::RangeTblFunction {
    fn to_tokens(&self, _e: &mut EventEmitter) {}
}
