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

  readonly formatter: Formatter = Formatter;
  private table: Table;
  private athleteId?: number;
  private readonly registrationsModel: JSONModel = new JSONModel();
  private readonly athleteModel: JSONModel = new JSONModel();

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("athleteRegistrationsTable") as Table;

    super.setViewModel(this.registrationsModel, "registrations");
    super.setViewModel(this.athleteModel, "athlete");

    super.getRouter()?.getRoute("athleteDetails")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  onNavBack(): void {
    super.navBack("athletes");
    delete this.athleteId;
  }

  onSelectionChange(oEvent: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = oEvent.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("registrations");
      const registration: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      registration.race._nav = { disabled: true, back: "athletes" };

      (super.getComponentModel("race") as JSONModel).setData(registration.race);
      super.navToRaceDetails(registration.race.id);
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
    return await Promise.all([super.updateJSONModel(this.registrationsModel, `/api/regattas/${regatta.id}/athletes/${this.athleteId}/registrations`, this.table),
    super.updateJSONModel(this.athleteModel, `/api/athletes/${this.athleteId}`)]);
  }
}