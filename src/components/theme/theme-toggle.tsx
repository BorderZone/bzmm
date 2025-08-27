import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTheme } from "./theme-provider";

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <Select value={theme} onValueChange={setTheme}>
      <SelectTrigger className="w-[180px] text-foreground">
        <SelectValue placeholder="Select theme" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="light" className="text-foreground">Light</SelectItem>
        <SelectItem value="dark" className="text-foreground">Dark</SelectItem>
        <SelectItem value="system" className="text-foreground">System</SelectItem>
      </SelectContent>
    </Select>
  );
}