return {
	"stevearc/oil.nvim",
	dependencies = { { "echasnovski/mini.icons", opts = {} } },
	keys = {
		{ "<leader>o", "<cmd>Oil --float<CR>", desc = "Open oil" },
	},
	opts = {
		view_options = {
			show_hidden = true,
		},
		keymaps = {
			-- TODO: Can we use shift-enter here?
			["<leader>sv"] = {
				"actions.select",
				opts = { vertical = true },
				desc = "Open the entry in a vertical split",
			},
			["<leader>sh"] = {
				"actions.select",
				opts = { horizontal = true },
				desc = "Open the entry in a horizontal split",
			},
		},
	},
}
