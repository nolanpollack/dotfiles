return {
	"mfussenegger/nvim-dap",
	keys = {
		{
			"<leader>rd",
			function()
				require("dap").continue()
			end,
			desc = "Run debug",
		},
		{
			"<leader>bp",
			function()
				require("dap").toggle_breakpoint()
			end,
			desc = "Toggle breakpoint",
		},
	},
}
