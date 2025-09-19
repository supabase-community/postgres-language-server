package tree_sitter_pgls_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_pgls "github.com/juleswritescode/tree-sitter-pgls/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_pgls.Language())
	if language == nil {
		t.Errorf("Error loading Postgres Language Server grammar")
	}
}
