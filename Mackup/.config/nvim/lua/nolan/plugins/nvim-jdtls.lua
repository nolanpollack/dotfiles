return {
	"mfussenegger/nvim-jdtls",
	config = function()
		local java_cmds = vim.api.nvim_create_augroup("java_cmds", { clear = true })
		local cache_vars = {}

		-- Determine root directory of project
		-- TODO: Make this work for repo
		local root_files = {
			".git",
			"gradlew",
			"build.gradle",
			"settings.gradle",
		}

		local features = {
			-- Set to true to enable codelens
			codelens = false,

			-- Set to true if nvim-dap, java-test, and java-debug-adapter are installed
			debugger = false,
		}

		-- Get paths needed to start LSP server
		local function get_jdtls_paths()
			if cache_vars.paths then
				return cache_vars.paths
			end

			local path = {}

			path.data_dir = vim.fn.stdpath("cache") .. "/nvim-jdtls"

			local jdtls_install = require("mason-registry").get_package("jdtls"):get_install_path()

			path.java_agent = jdtls_install .. "/lombok.jar"
			path.launcher_jar = vim.fn.glob(jdtls_install .. "/plugins/org.eclipse.equinox.launcher_*.jar")

			path.platform_config = jdtls_install .. "/config_linux"

			path.bundles = {}

			cache_vars.paths = path

			return path
		end

		local function jdtls_on_attach(client, bufnr)
			-- Executed everytime jdtls gets attached, set keymaps
		end

		local function jdtls_setup(event)
			-- Setup nvim-jdtls
			-- Executed everytime java file is opened

			local jdtls = require("jdtls")

			local path = get_jdtls_paths()
			local data_dir = path.data_dir .. "/" .. vim.fn.fnamemodify(vim.fn.getcwd(), ":p:h:t")

			if cache_vars.capabilities == nil then
				jdtls.extendedClientCapabilities.resolveAdditionalTextEditsSupport = true

				local ok_cmp, cmp_lsp = pcall(require, "cmp_nvim_lsp")
				cache_vars.capabilities = vim.tbl_deep_extend(
					"force",
					vim.lsp.protocol.make_client_capabilities(),
					ok_cmp and cmp_lsp.default_capabilities() or {}
				)
			end

			local cmd = {
				"java",

				"-Declipse.application=org.eclipse.jdt.ls.core.id1",
				"-Dosgi.bundles.defaultStartLevel=4",
				"-Declipse.product=org.eclipse.jdt.ls.core.product",
				"-Dlog.protocol=true",
				"-Dlog.level=ALL",
				"-javaagent:" .. path.java_agent,
				"-Xms1g",
				"--add-modules=ALL-SYSTEM",
				"--add-opens",
				"java.base/java.util=ALL-UNNAMED",
				"--add-opens",
				"java.base/java.lang=ALL-UNNAMED",

				-- 💀
				"-jar",
				path.launcher_jar,

				-- 💀
				"-configuration",
				path.platform_config,

				-- 💀
				"-data",
				data_dir,
			}

			local config = {
				cmd = cmd,
				root_dir = jdtls.setup.find_root(root_files),
				on_attach = jdtls_on_attach,
				init_options = {
					bundles = {},
					extendedClientCapabilities = {
						progressReportProvider = true,
					},
				},
			}

			jdtls.start_or_attach(config)
		end

		vim.api.nvim_create_autocmd("FileType", {
			group = java_cmds,
			pattern = { "java" },
			desc = "Setup jdtls",
			callback = jdtls_setup,
		})
	end,
}
