return {
	"zbirenbaum/copilot.lua",
	cmd = "Copilot",
	event = "InsertEnter",
	config = function()
		require("copilot").setup({
			panel = {
				autorefresh = true,
			},
			suggestion = {
				auto_trigger = true,
				keymap = {
					accept = "<CR>",
					dismiss = "<C-Space>",
				},
			},
		})
	end,
}
