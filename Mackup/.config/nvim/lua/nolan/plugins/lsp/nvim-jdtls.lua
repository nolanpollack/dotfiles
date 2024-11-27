return {
	"mfussenegger/nvim-jdtls",
	opts = {
		cmd = { vim.fn.expand("~/.local/share/nvim/mason/bin/jdtls") },
		root = { ".git", "gradlew", "pom.xml" },
		-- Settings for the jdtls server
		-- See https://github.com/eclipse-jdtls/eclipse.jdt.ls/wiki/Running-the-JAVA-LS-server-from-the-command-line#initialize-request for the options
		settings = {
			java = {
				inlayHints = { parameterNames = { enabled = "all" } },
				jdt = {
					ls = {
						androidSupport = {
							enabled = true,
						},
					},
				},
				signatureHelp = {
					enabled = true,
				},
			},
		},
	},
	ft = { "java" },
	config = function(_, opts)
        -- Set up debugger (add debug jar to bundles)
		local java_debug_path = require("mason-registry").get_package("java-debug-adapter"):get_install_path()
		local java_debug_jar = vim.fn.glob(java_debug_path .. "/extension/server/com.microsoft.java.debug.plugin-*.jar")

		local bundles = { java_debug_jar }

        -- Set up test runner (add all jars from java-test to bundles)
		local java_test_path = require("mason-registry").get_package("java-test"):get_install_path()
		vim.list_extend(bundles, vim.split(vim.fn.glob(java_test_path .. "/extension/server/*.jar", true), "\n"))

		local function attach_jdtls()
			require("jdtls").start_or_attach({
				cmd = opts.cmd,
				root_dir = vim.fs.root(0, opts.root),
				init_options = {
					bundles = bundles,
				},
				settings = opts.settings,
			})
		end
		-- Attach the jdtls for each java buffer. HOWEVER, this plugin loads
		-- depending on filetype, so this autocmd doesn't run for the first file.
		-- For that, we call directly below.
		vim.api.nvim_create_autocmd("FileType", {
			pattern = { "java" },
			callback = attach_jdtls,
		})

		attach_jdtls()
	end,
}
