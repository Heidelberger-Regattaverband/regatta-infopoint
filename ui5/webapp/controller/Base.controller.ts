import MyComponent from "de/regatta_hd/infoportal/Component";
import $ from "jquery";
import { LatLng } from "leaflet";
import Button from "sap/m/Button";
import MessageToast from "sap/m/MessageToast";
import EventBus from "sap/ui/core/EventBus";
import Controller from "sap/ui/core/mvc/Controller";
import View from "sap/ui/core/mvc/View";
import History from "sap/ui/core/routing/History";
import Router from "sap/ui/core/routing/Router";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Model$RequestFailedEventParameters } from "sap/ui/model/Model";
import { Routes, type RouteName } from "../model/Routes";

/**
 * Shared model access, translations, navigation, and asynchronous error reporting for view controllers.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class BaseController extends Controller {
  /**
   * Shows a message toast with the result of a data update operation.
   *
   * @param succeeded - Whether the data update completed successfully.
   */
  protected showDataUpdatedMessage(succeeded: boolean): void {
    if (succeeded) {
      this.showInfoMessageToast(this.i18n("msg.dataUpdated"));
    } else {
      this.showErrorMessageToast(this.i18n("msg.dataUpdateFailed"));
    }
  }

  /**
   * Handles promise rejections from work started by synchronous UI5 callbacks.
   * Logs failures and shows update feedback only while the view is still alive; resolved `false` values are not errors.
   *
   * @param task - Work already started by the caller.
   * @param errorContext - Prefix identifying the failed operation in the console.
   */
  protected runAsync(task: Promise<unknown>, errorContext: string): void {
    void task.catch((error: unknown) => {
      console.error(errorContext, error);
      const view = this.getView();
      if (view && !view.isDestroyed()) {
        this.showDataUpdatedMessage(false);
      }
    });
  }

  /**
   * Shows an info message toast with the given message.
   *
   * @param message - Text to display.
   */
  protected showInfoMessageToast(message: string): void {
    MessageToast.show(message);
    $(".sapMMessageToast").addClass("sapMMessageToastInfo");
  }

  /**
   * Shows an error message toast with the given message.
   *
   * @param message - Text to display.
   */
  protected showErrorMessageToast(message: string): void {
    MessageToast.show(message);
    $(".sapMMessageToast").addClass("sapMMessageToastDanger");
  }

  /**
   * Exposes the browser's secure-context status to view bindings.
   *
   * @returns Whether the browser considers the current page a secure context.
   */
  isSecureContext(): boolean {
    console.debug(`isSecureContext: ${globalThis.isSecureContext}`);
    return globalThis.isSecureContext;
  }

  /**
   * Reads the component's content density, falling back to cozy sizing if unavailable.
   *
   * @returns The component's density CSS class, or `sapUiSizeCozy` if no class is available.
   */
  protected getContentDensityClass(): string {
    return this.getComponent()?.getContentDensityClass() ?? "sapUiSizeCozy";
  }

  /**
   * Returns the owning component's event bus, or `undefined` when there is no owner.
   *
   * @returns The component-scoped event bus, or `undefined` if the controller has no owner component.
   */
  protected getEventBus(): EventBus | undefined {
    return super.getOwnerComponent()?.getEventBus();
  }

  /**
   * Reads a named component model as a JSON model.
   *
   * @param name - Model name registered on the owner component.
   * @returns The existing model, or a new, unattached empty model when no model is registered.
   */
  protected getComponentJSONModel(name: string): JSONModel {
    return (super.getOwnerComponent()?.getModel(name) as JSONModel) ?? new JSONModel();
  }

  /**
   * Returns the owning application component's router.
   *
   * @returns The router configured by the owner component's manifest.
   */
  protected getRouter(): Router {
    return this.getComponent().getRouter();
  }

  /**
   * Reads a named model available to the view, including models inherited from the component.
   *
   * @param name - Model name used by view bindings.
   * @returns The existing model, or a new, unattached empty model when no model is available.
   */
  protected getViewJSONModel(name: string): JSONModel {
    return (super.getView()?.getModel(name) as JSONModel) ?? new JSONModel();
  }

  /**
   * Registers a JSON model on this controller's view.
   *
   * @param model - Model to expose to bindings.
   * @param name - Named binding context for the model.
   * @returns The view, or `undefined` if it is unavailable.
   */
  protected setViewModel(model: JSONModel, name: string): View | undefined {
    return super.getView()?.setModel(model, name);
  }

  /**
   * Uses browser history when UI5 reports a previous hash; otherwise navigates to the fallback route.
   *
   * @param target - Named route used when there is no previous hash.
   */
  protected navBack(target: RouteName): void {
    const previousHash: string | undefined = History.getInstance().getPreviousHash();
    if (previousHash) {
      globalThis.history.back();
    } else {
      this.getRouter().navTo(target, {}, undefined, false /* history*/);
    }
  }

  /**
   * Translates the given key using the resource bundle of the component.
   *
   * @param key - Resource-bundle key.
   * @param args - Values substituted into translation placeholders.
   * @returns Translated text, or an empty string if no text is available.
   */
  protected i18n(key: string, args?: any[]): string {
    return this.getComponent()?.getResourceBundle().getText(key, args) ?? "";
  }

  /**
   * Loads backend data into a model and logs request failures without rethrowing them.
   *
   * @param model - Destination model updated by UI5's data loader.
   * @param url - Backend endpoint to request.
   * @param button - Optional control kept busy until the request settles.
   * @returns `true` after a successful load; `false` after a failed request.
   */
  protected async updateJSONModel(model: JSONModel, url: string, button?: Button): Promise<boolean> {
    button?.setBusy(true);
    try {
      await model.loadData(url);
      return true;
    } catch (error: any) {
      const params: Model$RequestFailedEventParameters = error as Model$RequestFailedEventParameters;
      console.error((params.statusCode ?? "") + ": " + params.statusText);
      return false;
    } finally {
      button?.setBusy(false);
    }
  }

  protected navToStartPage(): void {
    this.getRouter().navTo(Routes.StartPage);
  }

  navToRaces(): void {
    this.getRouter().navTo(Routes.Races);
  }

  /**
   * Opens a race's detail route.
   *
   * @param raceId - Backend race ID substituted into the route pattern.
   */
  protected navToRaceDetails(raceId: number): void {
    this.getRouter().navTo(Routes.RaceDetails, { raceId: raceId });
  }

  navToHeats(): void {
    this.getRouter().navTo(Routes.Heats);
  }

  /**
   * Opens a heat's detail route.
   *
   * @param heatId - Backend heat ID substituted into the route pattern.
   */
  protected navToHeatDetails(heatId: number): void {
    this.getRouter().navTo(Routes.HeatDetails, { heatId: heatId });
  }

  navToClubs(): void {
    this.getRouter().navTo(Routes.Clubs);
  }

  /**
   * Opens a club's detail route.
   *
   * @param clubId - Backend club ID substituted into the route pattern.
   */
  protected navToClubDetails(clubId: number): void {
    this.getRouter().navTo(Routes.ClubDetails, { clubId: clubId }, false /* history*/);
  }

  navToAthletes(): void {
    this.getRouter().navTo(Routes.Athletes);
  }

  /**
   * Opens the athlete list restricted to a club.
   *
   * @param clubId - Backend club ID passed to the club-specific athlete route.
   */
  protected navToClubAthletes(clubId: number): void {
    this.getRouter().navTo(Routes.ClubAthletes, { clubId: clubId });
  }

  /**
   * Opens an athlete's detail route.
   *
   * @param athleteId - Backend athlete ID substituted into the route pattern.
   */
  protected navToAthleteDetails(athleteId: number): void {
    this.getRouter().navTo(Routes.AthleteDetails, { athleteId: athleteId }, false /* history*/);
  }

  /**
   * Opens the map, optionally including a location in the route parameters.
   *
   * @param location - Coordinates to pass to the map route, when provided.
   */
  navToMap(location?: LatLng): void {
    let params: any = {};
    if (location) {
      params = { lat: location.lat, lng: location.lng };
    }
    this.getRouter().navTo(Routes.Map, params);
  }

  navToNotifications(): void {
    this.getRouter().navTo(Routes.Notifications);
  }

  /**
   * Returns filter-option data from the component's cached model; loading failures propagate to the caller.
   *
   * @returns A promise resolving to the active regatta's filter-option payload, not the JSON model itself.
   */
  protected async getFilters(): Promise<any> {
    return (await this.getComponent().getFilters()).getData();
  }

  /**
   * Returns active-regatta data from the component's cached model; loading failures propagate to the caller.
   *
   * @returns A promise resolving to the active-regatta payload, including its backend ID, not the JSON model itself.
   */
  protected async getActiveRegatta(): Promise<any> {
    return (await this.getComponent().getActiveRegatta()).getData();
  }

  /**
   * Accesses the owner using the application component's API.
   *
   * @returns The owner component cast to the application-specific component type.
   */
  private getComponent(): MyComponent {
    return super.getOwnerComponent() as MyComponent;
  }
}
