return {
	"sindrets/diffview.nvim",
	opts = {},
	keys = {
		{ "<leader>gdo", "<cmd>DiffviewOpen<CR>", desc = "Open diffview" },
		{ "<leader>gdc", "<cmd>DiffviewClose<CR>", desc = "Close diffview" },
		{ "<leader>gdh", "<cmd>DiffviewFileHistory<CR>", desc = "Open diffview for file history" },
		{ "<leader>gdf", "<cmd>DiffviewFileHistory %<CR>", desc = "View history for this file" },
		{
			"<leader>gdb",
			"<cmd>DiffviewOpen origin/HEAD...HEAD --imply-local<CR>",
			desc = "View changes made to this branch",
		},
	},
}
