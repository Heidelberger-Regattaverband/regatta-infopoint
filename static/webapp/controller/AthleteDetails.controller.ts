import Table from "sap/m/Table";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Button, { Button$PressEvent } from "sap/m/Button";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import ListBinding from "sap/ui/model/ListBinding";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AthleteDetailsController extends BaseController {

  formatter: Formatter = Formatter;
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
    this.loadRegistrationsModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding.filter(searchFilters);
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter({
          path: "crew/",
          test: function (crews: any[]) {
            for (let crew of crews) {
              const found = crew.athlete.firstName.toLowerCase().includes(query.toLowerCase())
                || crew.athlete.lastName.toLowerCase().includes(query.toLowerCase());
              if (found) {
                return true;
              }
            }
            return false;
          }
        }),
        new Filter("race/number", FilterOperator.Contains, query),
      ],
      and: false
    })]
  }

  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.athleteId = (event.getParameter("arguments") as any).athleteId;
    await Promise.all([this.loadRegistrationsModel(), this.loadAthleteModel()]);
  }

  private async loadAthleteModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    return await super.updateJSONModel(this.athleteModel, `/api/regattas/${regatta.id}/clubs/${this.athleteId}`);
  }

  private async loadRegistrationsModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    return await super.updateJSONModel(this.registrationsModel, `/api/regattas/${regatta.id}/clubs/${this.athleteId}/registrations`, this.table);
  }
}