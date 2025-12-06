-- TODO: Never really got this to work. Can I get a good flow going for at least react native?
return {
	{
		"mfussenegger/nvim-dap",
		keys = {
			{
				"<leader>dc",
				function()
					require("dap").continue()
				end,
				desc = "Debug continue",
			},
			{
				"<leader>db",
				function()
					require("dap").toggle_breakpoint()
				end,
				desc = "Toggle breakpoint",
			},
			{
				"<leader>do",
				function()
					require("dap").step_over()
				end,
				desc = "Debug step over",
			},
			{
				"<leader>di",
				function()
					require("dap").step_into()
				end,
				desc = "Debug step into",
			},
			{
				"<leader>duio",
				function()
					require("dapui").open()
				end,
				desc = "Open dap UI (Should make this automatic)",
			},
			{
				"<leader>duic",
				function()
					require("dapui").close()
				end,
				desc = "close dap UI (Should make this automatic)",
			},
		},
		config = function()
			local dapui = require("dapui")
			-- TODO: Load dapui lazily when dap loads
			dapui.setup()

			local sign = vim.fn.sign_define

			sign("DapBreakpoint", { text = "●", texthl = "DapBreakpoint", linehl = "", numhl = "" })
			sign("DapBreakpointCondition", { text = "●", texthl = "DapBreakpointCondition", linehl = "", numhl = "" })
			sign("DapLogPoint", { text = "◆", texthl = "DapLogPoint", linehl = "", numhl = "" })

			local dap, wk = require("dap"), require("which-key")
			wk.add({ "<leader>d", group = "Debug" })
			dap.listeners.before.attach.dapui_config = function()
				dapui.open()
			end
			-- dap.listeners.before.launch.dapui_config = function()
			-- 	dapui.open()
			-- end
			-- dap.listeners.before.event_terminated.dapui_config = function()
			-- 	dapui.close()
			-- end
			-- dap.listeners.before.event_exited.dapui_config = function()
			-- 	dapui.close()
			-- end
		end,
	},
	{ "rcarriga/nvim-dap-ui", dependencies = { "mfussenegger/nvim-dap", "nvim-neotest/nvim-nio" } },
	{
		"jbyuki/one-small-step-for-vimkind",
		dependencies = { "mfussenegger/nvim-dap" },
		filetype = {
			"lua",
		},
		config = function()
			local dap = require("dap")
			dap.configurations.lua = {
				{
					type = "nlua",
					request = "attach",
					name = "Attach to running Neovim instance",
				},
			}

			dap.adapters.nlua = function(callback, config)
				callback({ type = "server", host = "127.0.0.1", port = 8090 })
			end
			vim.keymap.set("n", "<leader>dl", function()
				require("osv").launch({ port = 8090 })
			end, { noremap = true })
		end,
	},
}
