import { selectTransferStep } from "@/features/transfer-form";
import { useAppSelector } from "@/store/hooks";
import {
  RecipientSearch,
  TransferAmountForm,
  TransferConfirmation,
} from "@/shared/ui/organisms";
import { MainTemplate } from "@/shared/ui/templates";

/**
 * Transfer wizard page with search, form, and confirmation steps.
 *
 * @returns Transfer screen.
 */
export function TransferPage() {
  const step = useAppSelector(selectTransferStep);

  return (
    <MainTemplate title="Send Money">
      {step === "search" ? <RecipientSearch /> : null}
      {step === "form" ? <TransferAmountForm /> : null}
      {step === "confirm" ? <TransferConfirmation /> : null}
    </MainTemplate>
  );
}
