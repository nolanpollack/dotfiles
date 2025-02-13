return {
	"nvim-tree/nvim-tree.lua",
	dependencies = "nvim-tree/nvim-web-devicons",
	keys = {
		{ "<leader>e", "<cmd>NvimTreeFindFileToggle<CR>", desc = "Toggle file explorer" },
	},
	init = function()
		-- recommended settings from nvim-tree documentation
		vim.g.loaded_netrw = 1
		vim.g.loaded_netrwPlugin = 1
	end,
	opts = {
		view = {
			width = 35,
			relativenumber = true,
		},
		-- change folder arrow icons
		renderer = {
			indent_markers = {
				enable = true,
			},
			icons = {
				glyphs = {
					folder = {
						arrow_closed = "", -- arrow when folder is closed
						arrow_open = "", -- arrow when folder is open
					},
				},
			},
		},
		-- disable window_picker for
		-- explorer to work well with
		-- window splits
		actions = {
			open_file = {
				window_picker = {
					enable = true,
				},
			},
		},
		diagnostics = {
			enable = true,
		},
		filters = {
			custom = { ".DS_Store" },
		},
		git = {
			ignore = false,
		},
		on_attach = function(bufnr)
			local api = require("nvim-tree.api")

			local function opts(desc)
				return { desc = "nvim-tree: " .. desc, buffer = bufnr, noremap = true, silent = true, nowait = true }
			end

			-- default mappings
			api.config.mappings.default_on_attach(bufnr)

			-- custom mappings
			vim.keymap.set("n", "?", api.tree.toggle_help, opts("Help"))
			vim.keymap.set("n", "sv", api.node.open.vertical, opts("Open: Vertical Split"))
			vim.keymap.set("n", "sh", api.node.open.horizontal, opts("Open: Horizontal Split"))
		end,
	},
}
