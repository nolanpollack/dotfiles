return {
	"isakbm/gitgraph.nvim",
	---@type I.GGConfig
	opts = {},
	keys = {
		{
			"<leader>gg",
			function()
				require("gitgraph").draw({}, { all = true, max_count = 5000 })
			end,
			desc = "GitGraph - Draw",
		},
	},
}
