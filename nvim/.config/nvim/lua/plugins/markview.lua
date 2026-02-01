return {
	"OXY2DEV/markview.nvim",
	dependencies = {
		"echasnovski/mini.icons",
	},
	ft = { "markdown" },
	keys = {
		{ "<leader>sm", "<cmd>Markview splitToggle<cr>", desc = "Toggle Markdown Preview Split" },
	},
	opts = {
		preview = {
			filetypes = { "markdown" },
			ignore_buftypes = {},
			enable = false,
		},
	},
}
