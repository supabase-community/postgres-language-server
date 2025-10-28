use pgls_query::{NodeRef, parse};

fn main() {
    let mut result = parse("SELECT * FROM users WHERE id IN (SELECT id FROM admins)").unwrap();

    // Immutable access
    {
        let stmts = result.stmts();
        let stmt = stmts.first().unwrap();

        // nodes() returns a Vec<NodeRef>
        let all_nodes = stmt.nodes();
        println!("Total nodes in AST: {}", all_nodes.len());

        // Can still iterate with iter()
        let select_count = stmt
            .iter()
            .filter(|n| matches!(n, NodeRef::SelectStmt(_)))
            .count();
        println!("Number of SELECT statements: {select_count}");
    }

    // Mutable access - no cloning needed!
    {
        let mut stmts = result.stmts_mut();
        if let Some(stmt) = stmts.first_mut() {
            // Now we can iterate mutably without cloning
            for mut_node in stmt.iter_mut() {
                // Modify nodes here if needed
                if let pgls_query::NodeMut::SelectStmt(_select) = mut_node {
                    println!("Found a SELECT statement to modify");
                    // You can modify _select here
                }
            }
        }
    }

    // Alternative: using root_mut() for single statement queries
    if let Some(root) = result.root_mut() {
        println!("Root node type: {:?}", std::mem::discriminant(root));
    }
}
