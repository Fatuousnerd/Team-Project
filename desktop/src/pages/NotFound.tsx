import { Compass } from "lucide-react";
import { Link } from "react-router";
import { Page } from "@/components/app/page";
import { Button } from "@/components/ui/button";
import {
	Empty,
	EmptyContent,
	EmptyDescription,
	EmptyHeader,
	EmptyMedia,
	EmptyTitle,
} from "@/components/ui/empty";

export default function NotFound() {
	return (
		<Page>
			<Empty className="rounded-xl border border-border/70 bg-card py-20">
				<EmptyHeader>
					<EmptyMedia variant="icon">
						<Compass />
					</EmptyMedia>
					<EmptyTitle>Page not found</EmptyTitle>
					<EmptyDescription>
						That route does not exist in the terminal. Head back to the
						dashboard to keep trading.
					</EmptyDescription>
				</EmptyHeader>
				<EmptyContent>
					<Button asChild size="sm">
						<Link to="/">Back to dashboard</Link>
					</Button>
				</EmptyContent>
			</Empty>
		</Page>
	);
}
