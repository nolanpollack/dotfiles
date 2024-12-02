return {
	"lewis6991/gitsigns.nvim",
	event = "BufRead",
	keys = {
		{ "<leader>gb", "<cmd>Gitsigns toggle_current_line_blame<CR>", desc = "Git blame" },
		{
			"ih",
			mode = { "o", "x" },
            ":<C-U>Gitsigns select_hunk<CR>",
			desc = "inner hunk",
		},
		{
			"<leader>gs",
			function()
				require("gitsigns").stage_hunk()
			end,
            desc = "Stage hunk",
		},
		{
			"<leader>gu",
			function()
				require("gitsigns").reset_hunk()
			end,
            desc = "Unstage hunk",
		},
	},
	opts = {},
}
