return {
	"zbirenbaum/copilot.lua",
	cmd = "Copilot",
	event = "InsertEnter",
	opts = {
		panel = {
			autorefresh = true,
		},
		suggestion = {
			auto_trigger = true,
			keymap = {
				accept = "<Tab>",
				dismiss = "<C-Space>",
			},
		},
	},
}
