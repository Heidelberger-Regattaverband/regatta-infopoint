import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Table from "sap/m/Table";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AthleteDetailsController extends BaseController {

  private static readonly ATHLETE_MODEL: string = "athlete";
  private static readonly ENTRIES_MODEL: string = "entries";

  readonly formatter: Formatter = Formatter;
  private table?: Table;
  private athleteId?: number;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("athleteEntriesTable") as Table;

    super.setViewModel(new JSONModel(), AthleteDetailsController.ENTRIES_MODEL);
    super.setViewModel(new JSONModel(), AthleteDetailsController.ATHLETE_MODEL);

    super.getRouter()?.getRoute("athleteDetails")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  onNavBack(): void {
    super.navBack("athletes");
    delete this.athleteId;
  }

  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(AthleteDetailsController.ENTRIES_MODEL);
      const entry: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      entry.race._nav = { disabled: true, back: "athletes" };

      super.getComponentJSONModel("race").setData(entry.race);
      super.navToRaceDetails(entry.race.id);
    }
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadData().then((succeeded: [boolean, boolean]) => {
      super.showDataUpdatedMessage(succeeded[0] && succeeded[1]);
    }).finally(() => source.setEnabled(true));
  }

  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.athleteId = (event.getParameter("arguments") as any).athleteId;
    await this.loadData();
  }

  private async loadData(): Promise<[boolean, boolean]> {
    const regatta: any = await super.getActiveRegatta();

    const athleteUrl: string = `/api/regattas/${regatta.id}/athletes/${this.athleteId}`;
    const entriesUrl: string = `/api/regattas/${regatta.id}/athletes/${this.athleteId}/entries`;

    const athleteModel: JSONModel = super.getViewJSONModel(AthleteDetailsController.ATHLETE_MODEL);
    const entriesModel: JSONModel = super.getViewJSONModel(AthleteDetailsController.ENTRIES_MODEL);

    return await Promise.all([super.updateJSONModel(entriesModel, entriesUrl), super.updateJSONModel(athleteModel, athleteUrl)]);
  }
}
