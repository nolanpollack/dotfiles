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
			"<leader>gs",
            ":'<,'>Gitsigns stage_hunk<CR>",
            desc = "Stage selected hunks",
            mode = "v",
		},
		{
			"<leader>gu",
			function()
				require("gitsigns").reset_hunk()
			end,
            desc = "Unstage hunk",
		},
        {
            "]g",
            function()
                require("gitsigns").nav_hunk('next')
            end,
            desc = "Next hunk",
        },
        {
            "[g",
            function()
                require("gitsigns").nav_hunk('prev')
            end,
            desc = "Previous hunk",
        },
	},
	opts = {},
}
