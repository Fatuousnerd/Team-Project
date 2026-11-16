import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { fmtMoney } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";

export function AccountSwitcher() {
	const accounts = useMarketStore((state) => state.accounts);
	const activeAccountId = useMarketStore((state) => state.activeAccountId);
	const setActiveAccount = useMarketStore((state) => state.setActiveAccount);
	const active = accounts.find((account) => account.id === activeAccountId);

	return (
		<Select value={activeAccountId} onValueChange={setActiveAccount}>
			<SelectTrigger
				size="sm"
				className="w-[168px] border-border/80 bg-transparent"
				aria-label="Active account"
			>
				<SelectValue placeholder="Select account">{active?.name}</SelectValue>
			</SelectTrigger>
			<SelectContent align="end">
				{accounts.map((account) => (
					<SelectItem key={account.id} value={account.id}>
						{account.name} · {fmtMoney(account.balance, { whole: true })}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
