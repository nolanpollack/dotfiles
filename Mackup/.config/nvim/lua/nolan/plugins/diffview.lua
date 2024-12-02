return {
	"sindrets/diffview.nvim",
	opts = {
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
		{ "<leader>gd", "<cmd>DiffviewOpen<CR>", desc = "Open diffview" },
		{ "<leader>gl", "<cmd>DiffviewFileHistory<CR>", desc = "Git log" },
		{ "<leader>gh", "<cmd>DiffviewFileHistory %<CR>", desc = "View history for this file" },
		{
			"<leader>gr",
			"<cmd>DiffviewOpen origin/HEAD...HEAD --imply-local<CR>",
			desc = "View changes made to this branch",
		},
	},
    setup = function()
        require("diffview").setup()
    end,
}
