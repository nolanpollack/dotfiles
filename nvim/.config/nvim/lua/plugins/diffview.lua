return {
	"sindrets/diffview.nvim",
	opts = {
		-- TODO: Better keymaps while in editor (next file, previous file, stage, etc, go to panel)
		keymaps = {
			view = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			diff1 = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			diff2 = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			diff3 = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			diff4 = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			file_panel = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			file_history_panel = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			option_panel = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
			help_panel = {
				["<leader>gd"] = "<cmd>DiffviewClose<CR>",
			},
		},
	},
	keys = {
		-- TODO: Command to open prompt to type in what I want to diffview. Even better if it autofills refs
		{ "<leader>gd", "<cmd>DiffviewOpen<CR>", desc = "Open diffview" },
		{ "<leader>gl", "<cmd>DiffviewFileHistory<CR>", desc = "Git log" },
		{ "<leader>gh", "<cmd>DiffviewFileHistory %<CR>", desc = "View history for this file" },
		{
			"<leader>gr",
			"<cmd>DiffviewOpen origin/HEAD...HEAD --imply-local<CR>",
			desc = "View changes made to this branch",
		},
		{
			"<leader>gc",
			function()
				local line = vim.fn.line(".")
				local file = vim.fn.expand("%")
				vim.cmd("DiffviewFileHistory -L" .. line .. "," .. line .. ":" .. file)
			end,
			desc = "View changes made under the cursor",
		},
		{
			"<leader>gc",
			":'<,'>DiffviewFileHistory<CR>",
			desc = "View changes made to the current selection",
			mode = "v",
		},
	},
}
