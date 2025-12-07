return {
	"folke/trouble.nvim",
	dependencies = { "nvim-tree/nvim-web-devicons", "folke/todo-comments.nvim" },
	keys = {
		{ "<leader>xx", "<cmd>Trouble diagnostics toggle<cr>", desc = "Diagnostics (Trouble)" },
		{
			"<leader>xl",
			"<cmd>Trouble lsp toggle focus=false win.position=right<cr>",
			desc = "LSP Definitions / references / ... (Trouble)",
		},
		{
			"<leader>xs",
			"<cmd>Trouble lsp_document_symbols toggle focus=false win.position=right<cr>",
			desc = "LSP Document symbols (Trouble)",
		},
		{
			"gr",
			"<cmd>Trouble goto_references<cr>",
			desc = "Go to References (Trouble)",
		},
		{
			"gi",
			"<cmd>Trouble lsp_implementations toggle focus=false win.position=right<cr>",
			desc = "Go to implementations (Trouble)",
		},
		{ "<leader>xt", "<cmd>Trouble todo toggle<cr>", desc = "Todo comments (Trouble)" },
	},
	opts = {
		modes = {
			goto_references = {
				mode = "lsp_references",
				focus = true,
				preview = {
					type = "split",
					relative = "editor",
					position = "right",
					size = 0.5,
				},
				keys = {
					["<cr>"] = "jump_close",
				},
			},
		},
	},
}
