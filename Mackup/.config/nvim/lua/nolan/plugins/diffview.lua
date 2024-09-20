return {
	"sindrets/diffview.nvim",
	config = function()
		require("diffview").setup({})
		vim.keymap.set("n", "<leader>gdo", "<cmd>DiffviewOpen<CR>", { desc = "Open diffview" })
		vim.keymap.set("n", "<leader>gdc", "<cmd>DiffviewClose<CR>", { desc = "Close diffview" })
		vim.keymap.set("n", "<leader>gdh", "<cmd>DiffviewFileHistory<CR>", { desc = "Open diffview for file history" })
		vim.keymap.set("n", "<leader>gdf", "<cmd>DiffviewFileHistory %<CR>", { desc = "View history for this file" })
		vim.keymap.set(
			"n",
			"<leader>gdb",
			"<cmd>DiffviewOpen origin/HEAD...HEAD --imply-local<CR>",
			{ desc = "View changes made to this branch" }
		)
	end,
}
